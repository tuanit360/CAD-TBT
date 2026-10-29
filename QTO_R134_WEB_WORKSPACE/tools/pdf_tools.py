#!/usr/bin/env python3
"""Convert a PDF page to a measurement image, and optionally apply reviewed marks."""
import argparse, json, math
from pathlib import Path
import fitz

def run():
    ap=argparse.ArgumentParser()
    sp=ap.add_subparsers(dest='command',required=True)
    render=sp.add_parser('render');render.add_argument('pdf');render.add_argument('--page',type=int,default=1);render.add_argument('--out',required=True);render.add_argument('--dpi',type=int,default=160)
    mark=sp.add_parser('mark');mark.add_argument('pdf');mark.add_argument('project');mark.add_argument('--page',type=int,default=1);mark.add_argument('--out',required=True)
    a=ap.parse_args()
    with fitz.open(a.pdf) as pdf:
        if a.page<1 or a.page>len(pdf):raise ValueError('Trang PDF ngoài phạm vi')
        page=pdf[a.page-1]
        if a.command=='render':
            if not 72<=a.dpi<=300:raise ValueError('DPI phải từ 72 đến 300')
            page.get_pixmap(matrix=fitz.Matrix(a.dpi/72,a.dpi/72),alpha=False).save(a.out)
            print('PNG',a.out,'page',a.page,'dpi',a.dpi)
        else:
            project=json.loads(Path(a.project).read_text(encoding='utf-8'))
            if project.get('schema')!='tbt360-qto-v1':raise ValueError('Sai schema')
            # Browser canvas measurements are in rendered-image pixels; require original DPI to map accurately.
            dpi=float(project.get('pdf_render_dpi',0))
            if not 72<=dpi<=300:raise ValueError('Dự án phải có pdf_render_dpi để xuất PDF chính xác. Không được giả định tỷ lệ.')
            factor=72/dpi
            for measurement in project.get('measurements',[]):
                if measurement.get('source')!='PDF/Image' or int(measurement.get('page',1))!=a.page:continue
                points=measurement.get('points',[])
                if not points or len(points)>20000 or any(not isinstance(p,list) or len(p)!=2 or not all(isinstance(v,(float,int)) and math.isfinite(v) for v in p) for p in points):raise ValueError('Điểm không hợp lệ')
                path=[fitz.Point(p[0]*factor,p[1]*factor) for p in points]
                if measurement['kind']=='count':
                    if hasattr(page,'add_circle_annot'):
                        p=path[0];ann=page.add_circle_annot(fitz.Rect(p.x-4,p.y-4,p.x+4,p.y+4));ann.update()
                    continue
                pairs=list(zip(path,path[1:]))
                if measurement['kind'] in ('area','perimeter'):pairs.append((path[-1],path[0]))
                for start,end in pairs:
                    an=page.add_line_annot(start,end);an.set_colors(stroke=(.84,.14,.15));an.set_border(width=1.3);an.update()
            pdf.save(a.out,garbage=4,deflate=True)
            print('Annotated PDF',a.out)
if __name__=='__main__':run()
