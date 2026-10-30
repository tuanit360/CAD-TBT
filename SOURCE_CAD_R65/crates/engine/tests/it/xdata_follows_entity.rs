//! Other applications' xdata is part of its object, as in AutoCAD: COPY, MIRROR, ARRAY and paste
//! give the copies the same xdata, so it is saved on them too; the pieces of an exploded object
//! don't keep it.

use cadcraft_engine::Session;
use cadcraft_engine::doc::{Drawing, EntityKind, Handle, RawTag};
use serde_json::json;

/// A LINE with a hyperlink (`PE_URL`) and another application's data, and an LWPOLYLINE with
/// the same, read from DXF.
const DXF: &str = "0\nSECTION\n2\nENTITIES\n\
0\nLINE\n5\n2A\n8\n0\n10\n0\n20\n0\n30\n0\n11\n10\n21\n0\n31\n0\n\
1001\nPE_URL\n1000\nhttps://example.com/door-7\n1002\n{\n1000\nDoor 7\n1002\n}\n\
1001\nACME_TAGS\n1000\nD-07\n1070\n3\n1040\n2.5\n\
0\nLWPOLYLINE\n5\n2B\n8\n0\n90\n3\n70\n0\n10\n0\n20\n5\n10\n5\n20\n5\n10\n5\n20\n10\n\
1001\nACME_TAGS\n1000\nP-01\n\
0\nENDSEC\n0\nEOF\n";

fn tags(xdata: &[RawTag]) -> Vec<(i32, String)> {
    xdata.iter().map(|t| (t.code, t.value.trim().to_string())).collect()
}

fn session() -> (Session, Handle, Handle) {
    let d = cadcraft_io::read_dxf(DXF.as_bytes()).unwrap();
    let mut s = Session::empty();
    s.open_drawing(d, "xdata", None);
    let doc = s.doc().unwrap();
    let find = |line: bool| doc.model.iter().find(|e| matches!(e.kind, EntityKind::Line(_)) == line).map(|e| e.handle).unwrap();
    let (l, p) = (find(true), find(false));
    (s, l, p)
}

fn xdata_of(d: &Drawing, h: Handle) -> Vec<(i32, String)> {
    tags(&d.entity(h).unwrap().common.xdata)
}

fn new_handles(s: &Session, before: &[Handle]) -> Vec<Handle> {
    s.doc().unwrap().model.iter().map(|e| e.handle).filter(|h| !before.contains(h)).collect()
}

fn model_handles(s: &Session) -> Vec<Handle> {
    s.doc().unwrap().model.iter().map(|e| e.handle).collect()
}

fn add_bim(s:&mut Session,h:Handle,floor:u64,discipline:&str,owner:&str,revision:u64) {
    s.doc_mut().unwrap().modify_entity(h,|e|e.common.xdata.extend([
        RawTag{code:1001,value:"TBT360_BIM".into()},
        RawTag{code:1000,value:format!("floor={floor};discipline={discipline};owner={owner};revision={revision};entity={}",h.hex())},
    ])).unwrap();
}

fn bim_payload(s:&Session,h:Handle)->String {
    let e=s.doc().unwrap().entity(h).unwrap();
    let at=e.common.xdata.iter().position(|t|t.code==1001&&t.value=="TBT360_BIM").unwrap();
    e.common.xdata.get(at+1).unwrap().value.clone()
}

#[test]
fn reading_puts_xdata_on_the_entity() {
    let (s, l, p) = session();
    let d = s.doc().unwrap();
    let x = xdata_of(d, l);
    assert_eq!(x.first(), Some(&(1001, "PE_URL".to_string())));
    assert!(x.contains(&(1001, "ACME_TAGS".to_string())) && x.contains(&(1000, "D-07".to_string())));
    assert_eq!(xdata_of(d, p), vec![(1001, "ACME_TAGS".to_string()), (1000, "P-01".to_string())]);
    assert!(d.foreign.entities.values().all(|x| x.xdata.is_empty()), "entity xdata is not kept twice");
}

#[test]
fn copies_keep_the_xdata() {
    let (mut s, l, _) = session();
    let want = xdata_of(s.doc().unwrap(), l);

    let before = model_handles(&s);
    s.execute("copy", &json!({ "handles": [l.hex()], "delta": [0, 20] })).unwrap();
    let copied = new_handles(&s, &before);
    assert_eq!(copied.len(), 1);
    assert_eq!(xdata_of(s.doc().unwrap(), copied[0]), want, "COPY");

    let before = model_handles(&s);
    s.execute("mirror", &json!({ "handles": [l.hex()], "p1": [0, -1], "p2": [10, -1] })).unwrap();
    let mirrored = new_handles(&s, &before);
    assert_eq!(mirrored.len(), 1);
    assert_eq!(xdata_of(s.doc().unwrap(), mirrored[0]), want, "MIRROR");

    let before = model_handles(&s);
    s.execute("arrayrect", &json!({ "handles": [l.hex()], "rows": 1, "cols": 3, "colSpacing": 20, "associative": false })).unwrap();
    let arrayed = new_handles(&s, &before);
    assert_eq!(arrayed.len(), 2);
    for h in arrayed {
        assert_eq!(xdata_of(s.doc().unwrap(), h), want, "ARRAYRECT");
    }

    s.execute("copyclip", &json!({ "handles": [l.hex()] })).unwrap();
    let before = model_handles(&s);
    s.execute("pasteclip", &json!({ "at": [100, 100] })).unwrap();
    let pasted = new_handles(&s, &before);
    assert_eq!(pasted.len(), 1);
    assert_eq!(xdata_of(s.doc().unwrap(), pasted[0]), want, "PASTECLIP");

    // Undo takes the copy away; the original keeps its xdata.
    s.execute("u", &json!({})).unwrap();
    assert!(s.doc().unwrap().entity(pasted[0]).is_none());
    assert_eq!(xdata_of(s.doc().unwrap(), l), want);
}

#[test]
fn tbt360_bim_copies_get_independent_identity() {
    let (mut s,l,_)=session();
    s.doc_mut().unwrap().modify_entity(l,|e|e.common.xdata.extend([
        RawTag{code:1001,value:"TBT360_BIM".into()},
        RawTag{code:1000,value:format!("floor=2;discipline=ELEC;owner=u1;revision=7;entity={}",l.hex())},
    ])).unwrap();

    let payload=|s:&Session,h:Handle|{
        let e=s.doc().unwrap().entity(h).unwrap();
        let at=e.common.xdata.iter().position(|t|t.code==1001&&t.value=="TBT360_BIM").unwrap();
        e.common.xdata.get(at+1).unwrap().value.clone()
    };
    let assert_copy=|s:&Session,h:Handle|{
        let p=payload(s,h);
        assert!(p.contains("floor=2"));
        assert!(p.contains("discipline=ELEC"));
        assert!(p.contains("owner=u1"));
        assert!(p.contains("revision=1"));
        assert!(p.contains(&format!("entity={}",h.hex())));
    };

    let before=model_handles(&s);
    s.execute("copy",&json!({"handles":[l.hex()],"delta":[0,20]})).unwrap();
    let copied=new_handles(&s,&before);
    assert_eq!(copied.len(),1);
    assert_copy(&s,copied[0]);

    let before=model_handles(&s);
    s.execute("mirror",&json!({"handles":[l.hex()],"p1":[0,-1],"p2":[10,-1]})).unwrap();
    let mirrored=new_handles(&s,&before);
    assert_eq!(mirrored.len(),1);
    assert_copy(&s,mirrored[0]);

    let before=model_handles(&s);
    s.execute("arrayrect",&json!({"handles":[l.hex()],"rows":1,"cols":3,"colSpacing":20,"associative":false})).unwrap();
    let arrayed=new_handles(&s,&before);
    assert_eq!(arrayed.len(),2);
    for h in arrayed { assert_copy(&s,h); }

    let original=payload(&s,l);
    assert!(original.contains("revision=7"));
    assert!(original.contains(&format!("entity={}",l.hex())));
}

#[test]
fn a_copy_saves_its_xdata() {
    let (mut s, l, _) = session();
    s.execute("copy", &json!({ "handles": [l.hex()], "delta": [0, 20] })).unwrap();
    let text = cadcraft_io::write_dxf(s.doc().unwrap());
    let groups: Vec<&str> = text.lines().map(str::trim).collect();
    let count = |code: &str, value: &str| groups.chunks(2).filter(|g| *g == [code, value]).count();
    assert_eq!(count("1001", "PE_URL"), 2, "on the line and on its copy");
    assert_eq!(count("1000", "https://example.com/door-7"), 2);
    assert_eq!((count("2", "PE_URL"), count("2", "ACME_TAGS")), (1, 1), "the applications stay registered");
    let back = cadcraft_io::read_dxf(text.as_bytes()).unwrap();
    let lines: Vec<_> = back.model.iter().filter(|e| matches!(e.kind, EntityKind::Line(_))).collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(tags(&lines[0].common.xdata), tags(&lines[1].common.xdata));
}

#[test]
fn trim_pieces_get_new_bim_identities() {
    let mut s=Session::new();
    let target=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let e1=Handle::parse_hex(s.execute("line",&json!({"points":[[3,-5],[3,5]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let e2=Handle::parse_hex(s.execute("line",&json!({"points":[[7,-5],[7,5]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,target,2,"ELEC","u1",7);
    let r=s.execute("trim",&json!({"handle":target.hex(),"pick":[5,0],"edges":[e1.hex(),e2.hex()]})).unwrap();
    let hs=r["handles"].as_array().unwrap().iter().filter_map(|v|v.as_str().and_then(Handle::parse_hex)).collect::<Vec<_>>();
    assert_eq!(hs.len(),2);
    assert!(s.doc().unwrap().entity(target).is_none());
    for h in hs {
        let p=bim_payload(&s,h);
        assert!(p.contains("floor=2"));
        assert!(p.contains("discipline=ELEC"));
        assert!(p.contains("owner=u1"));
        assert!(p.contains("revision=1"));
        assert!(p.contains(&format!("entity={}",h.hex())));
    }
}

#[test]
fn extend_keeps_bim_identity_and_bumps_revision() {
    let mut s=Session::new();
    let target=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let edge=Handle::parse_hex(s.execute("line",&json!({"points":[[20,-5],[20,5]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,target,3,"STRUCT","u2",4);
    s.execute("extend",&json!({"handle":target.hex(),"pick":[10,0],"edges":[edge.hex()]})).unwrap();
    let p=bim_payload(&s,target);
    assert!(p.contains("floor=3"));
    assert!(p.contains("discipline=STRUCT"));
    assert!(p.contains("owner=u2"));
    assert!(p.contains("revision=5"));
    assert!(p.contains(&format!("entity={}",target.hex())));
}

#[test]
fn offset_gets_new_tbt360_bim_identity() {
    let (mut s,l,_)=session();
    s.doc_mut().unwrap().modify_entity(l,|e|e.common.xdata.extend([
        RawTag{code:1001,value:"TBT360_BIM".into()},
        RawTag{code:1000,value:format!("floor=2;discipline=ELEC;owner=u1;revision=7;entity={}",l.hex())},
    ])).unwrap();
    let r=s.execute("offset",&json!({"handle":l.hex(),"distance":2.0,"side":[0.0,5.0]})).unwrap();
    let nh=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let p=xdata_of(s.doc().unwrap(),nh);
    let payload=p.iter().find(|(code,value)|*code==1000&&value.contains("floor=2")).unwrap().1.clone();
    assert!(payload.contains("discipline=ELEC"));
    assert!(payload.contains("owner=u1"));
    assert!(payload.contains("revision=1"));
    assert!(payload.contains(&format!("entity={}",nh.hex())));
    let original=xdata_of(s.doc().unwrap(),l);
    assert!(original.iter().any(|(_,v)|v.contains("revision=7")));
}

#[test]
fn exploded_bim_pieces_keep_scope_with_unique_identity_only() {
    let (mut s,_,p)=session();
    s.doc_mut().unwrap().modify_entity(p,|e|e.common.xdata.extend([
        RawTag{code:1001,value:"TBT360_BIM".into()},
        RawTag{code:1000,value:format!("floor=3;discipline=STRUCT;owner=u2;revision=4;entity={}",p.hex())},
    ])).unwrap();
    let before=model_handles(&s);
    s.execute("explode",&json!({"handles":[p.hex()]})).unwrap();
    let pieces=new_handles(&s,&before);
    assert_eq!(pieces.len(),2);
    for h in pieces {
        let tags=xdata_of(s.doc().unwrap(),h);
        assert!(!tags.iter().any(|(_,v)|v=="ACME_TAGS"||v=="P-01"),"foreign xdata stays removed on own EXPLODE pieces");
        let payload=tags.iter().find(|(code,value)|*code==1000&&value.contains("floor=3")).unwrap().1.clone();
        assert!(payload.contains("discipline=STRUCT"));
        assert!(payload.contains("owner=u2"));
        assert!(payload.contains("revision=1"));
        assert!(payload.contains(&format!("entity={}",h.hex())));
    }
}

#[test]
fn exploded_pieces_do_not_keep_the_xdata() {
    let (mut s, _, p) = session();
    let before = model_handles(&s);
    s.execute("explode", &json!({ "handles": [p.hex()] })).unwrap();
    let pieces = new_handles(&s, &before);
    assert_eq!(pieces.len(), 2);
    for h in pieces {
        assert!(s.doc().unwrap().entity(h).unwrap().common.xdata.is_empty());
    }
}
