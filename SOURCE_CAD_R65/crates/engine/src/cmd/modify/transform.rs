use super::*;

// ---------------- core operations ----------------

pub(crate) fn refresh_tbt360_bim_copy_identity(common:&mut cadcraft_doc::Common,new_handle:Handle) {
    let mut in_bim=false;
    for tag in &mut common.xdata {
        if tag.code==1001 {
            in_bim=tag.value.eq_ignore_ascii_case("TBT360_BIM");
            continue;
        }
        if in_bim && tag.code==1000 {
            let mut parts=tag.value.split(';').map(str::to_string).collect::<Vec<_>>();
            let mut has_entity=false;
            let mut has_revision=false;
            for part in &mut parts {
                if part.starts_with("entity=") {
                    *part=format!("entity={}",new_handle.hex());
                    has_entity=true;
                } else if part.starts_with("revision=") {
                    *part="revision=1".into();
                    has_revision=true;
                }
            }
            if !has_entity { parts.push(format!("entity={}",new_handle.hex())); }
            if !has_revision { parts.push("revision=1".into()); }
            tag.value=parts.join(";");
            break;
        }
    }
}

pub(crate) fn transform_entities(s: &mut Session, hs: &[Handle], m: &Mat3, copy: bool) -> Result<Vec<Handle>> {
    let d = s.doc_mut()?;
    let mut out = Vec::new();
    for h in hs {
        let Some(sp) = d.space_of(*h) else { continue };
        if copy {
            let Some(e) = d.entity(*h).map(|e| (**e).clone()) else { continue };
            let nh = d.new_handle();
            let mut ne = Entity { handle: nh, ..e };
            refresh_tbt360_bim_copy_identity(&mut ne.common,nh);
            ne.kind.transform(m);
            if let Some(st) = d.space_mut(&sp) {
                st.push(ne);
                out.push(nh);
            }
        } else {
            let locked = d.entity(*h).and_then(|e| d.layer(&e.common.layer)).is_some_and(|l| l.locked);
            if locked {
                continue;
            }
            d.modify_entity(*h, |e| e.kind.transform(m))?;
            out.push(*h);
        }
    }
    Ok(out)
}

pub(super) fn erase(s: &mut Session, hs: &[Handle]) -> Result<usize> {
    let d = s.doc_mut()?;
    let mut n = 0;
    for h in hs {
        let locked = d.entity(*h).and_then(|e| d.layer(&e.common.layer)).is_some_and(|l| l.locked);
        if !locked && d.remove_entity(*h).is_some() {
            n += 1;
        }
    }
    s.set_selection(Vec::new());
    Ok(n)
}

pub(super) fn run_erase(s: &mut Session, p: &Value) -> Result<Value> {
    let hs = targets(s, p)?;
    let n = erase(s, &hs)?;
    Ok(json!({ "erased": n }))
}

pub(super) fn delta_of(cmd: &str, p: &Value) -> Result<Vec2> {
    if let Some(d) = point_param(p, "delta") {
        return Ok(d);
    }
    let a = point_req(cmd, p, "from")?;
    let b = point_req(cmd, p, "to")?;
    Ok(b - a)
}

pub(super) fn run_move(s: &mut Session, p: &Value) -> Result<Value> {
    let hs = targets(s, p)?;
    let d = delta_of("move", p)?;
    let moved = transform_entities(s, &hs, &Mat3::translate(d), false)?;
    Ok(json!({ "moved": moved.len() }))
}

pub(super) fn run_copy(s: &mut Session, p: &Value) -> Result<Value> {
    let hs = targets(s, p)?;
    let d = delta_of("copy", p)?;
    let n = p.get("count").and_then(Value::as_u64).unwrap_or(1).clamp(1, 10_000);
    let mut out = Vec::new();
    for k in 1..=n {
        out.extend(transform_entities(s, &hs, &Mat3::translate(d * k as f64), true)?.into_iter().map(|h| h.hex()));
    }
    Ok(json!({ "handles": out }))
}

pub(super) fn run_rotate(s: &mut Session, p: &Value) -> Result<Value> {
    let hs = targets(s, p)?;
    let base = point_req("rotate", p, "base")?;
    let a = f64_req("rotate", p, "angle")?.to_radians();
    let r = transform_entities(s, &hs, &Mat3::rotate_about(base, a), bool_or(p, "copy", false))?;
    Ok(json!({ "handles": r.iter().map(|h| h.hex()).collect::<Vec<_>>() }))
}

pub(super) fn run_scale(s: &mut Session, p: &Value) -> Result<Value> {
    let hs = targets(s, p)?;
    let base = point_req("scale", p, "base")?;
    let f = f64_req("scale", p, "factor")?;
    if f <= 0.0 {
        return Err(bad("scale", "factor must be positive"));
    }
    let r = transform_entities(s, &hs, &Mat3::scale_about(base, f), bool_or(p, "copy", false))?;
    Ok(json!({ "handles": r.iter().map(|h| h.hex()).collect::<Vec<_>>() }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tbt360_copy_identity_keeps_scope_and_resets_identity_revision() {
        let mut common=cadcraft_doc::Common::default();
        common.xdata=vec![
            cadcraft_doc::RawTag{code:1001,value:"TBT360_BIM".into()},
            cadcraft_doc::RawTag{code:1000,value:"floor=2;discipline=ELEC;owner=u1;revision=9;entity=2A".into()},
        ];
        refresh_tbt360_bim_copy_identity(&mut common,Handle(0x55));
        assert_eq!(common.xdata[1].value,"floor=2;discipline=ELEC;owner=u1;revision=1;entity=55");
    }

    #[test]
    fn non_bim_xdata_is_untouched() {
        let mut common=cadcraft_doc::Common::default();
        common.xdata=vec![
            cadcraft_doc::RawTag{code:1001,value:"ACME".into()},
            cadcraft_doc::RawTag{code:1000,value:"revision=9;entity=2A".into()},
        ];
        let before=common.xdata.clone();
        refresh_tbt360_bim_copy_identity(&mut common,Handle(0x55));
        assert_eq!(common.xdata,before);
    }
}

pub(super) fn run_mirror(s: &mut Session, p: &Value) -> Result<Value> {
    let hs = targets(s, p)?;
    let a = point_req("mirror", p, "p1")?;
    let b = point_req("mirror", p, "p2")?;
    if a.near(b, 1e-12) {
        return Err(bad("mirror", "mirror line points coincide"));
    }
    let erase_src = bool_or(p, "erase", false);
    let r = transform_entities(s, &hs, &Mat3::mirror(a, b), !erase_src)?;
    Ok(json!({ "handles": r.iter().map(|h| h.hex()).collect::<Vec<_>>() }))
}
