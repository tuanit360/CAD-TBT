#!/usr/bin/env python3
"""DXF quantities with explicit INSUNITS, arcs and bulge handling through ezdxf."""
import argparse,csv,math
from pathlib import Path
import ezdxf
from ezdxf import units
from ezdxf.path import make_path

def takeoff(path,out):
    doc=ezdxf.readfile(path)
    code=doc.header.get('$INSUNITS',0)
    if code in (0,):raise ValueError('DXF không có đơn vị ($INSUNITS=0): yêu cầu hiệu chỉnh thủ công')
    factor=units.conversion_factor(code,units.M)
    if not math.isfinite(factor) or factor<=0:raise ValueError('Đơn vị không hợp lệ')
    result=[]
    for entity in doc.modelspace():
        kind=entity.dxftype();value=None;unit='m'
        if kind=='LINE':value=(entity.dxf.end-entity.dxf.start).magnitude*factor
        elif kind=='CIRCLE':value=math.pi*(entity.dxf.radius*factor)**2;unit='m²'
        elif kind=='ARC':value=entity.dxf.radius*factor*math.radians((entity.dxf.end_angle-entity.dxf.start_angle)%360)
        elif kind in ('LWPOLYLINE','POLYLINE'):
            try:
                path_obj=make_path(entity)
                # Curves flattened with documented max deviation (source units); approximate, flag in status.
                points=[(v.x,v.y) for v in path_obj.flattening(distance=max(.001,.002/factor))]
                if len(points)<2:continue
                if kind=='LWPOLYLINE':closed=bool(entity.closed)
                else:closed=bool(entity.is_closed)
                if closed and points[0]!=points[-1]:points.append(points[0])
                length=sum(math.dist(a,b) for a,b in zip(points,points[1:]))*factor
                if closed and len(points)>3:
                    value=abs(sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(points,points[1:])))*.5*factor**2;unit='m²'
                else:value=length
            except Exception:continue
        if value is None:continue
        if not math.isfinite(value) or value<0:raise ValueError('Khối lượng sai')
        result.append((entity.dxf.handle,entity.dxf.layer,kind,f'{value:.6f}',unit,'GEOMETRY_APPROX' if kind in ('LWPOLYLINE','POLYLINE') else 'GEOMETRY_EXACT'))
    with open(out,'w',encoding='utf-8-sig',newline='') as f:
        w=csv.writer(f);w.writerow(['source_id','classification','measure_type','quantity','unit','status']);w.writerows(result)
    print('units',code,'metres_per_unit',factor,'measurements',len(result),'output',out)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('dxf');p.add_argument('--out',required=True);a=p.parse_args();takeoff(a.dxf,a.out)
