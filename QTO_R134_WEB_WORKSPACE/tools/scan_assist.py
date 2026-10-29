"""Suggests line segments from PDF scan/images for review; never produces certified QTO."""
import argparse
import json
from pathlib import Path
import cv2
import fitz
import numpy as np


def analyse(source: Path, out_dir: Path, max_pages=20):
    out_dir.mkdir(parents=True, exist_ok=True)
    if source.suffix.lower() == '.pdf':
        pdf = fitz.open(source)
        if len(pdf)>max_pages: raise ValueError('PDF vượt giới hạn trang')
        imgs = []
        for page in pdf:
            pix = page.get_pixmap(matrix=fitz.Matrix(1.5,1.5), alpha=False)
            imgs.append(np.frombuffer(pix.samples,dtype=np.uint8).reshape(pix.height,pix.width,3)[:,:,::-1].copy())
    else:
        img=cv2.imread(str(source))
        if img is None: raise ValueError('Không đọc được ảnh')
        imgs=[img]
    result={'source':source.name,'status':'CANDIDATE_ONLY_NOT_VERIFIED', 'pages':[]}
    for idx,img in enumerate(imgs):
        if img.shape[0]*img.shape[1]>30_000_000: raise ValueError('Ảnh quá lớn')
        gray=cv2.cvtColor(img,cv2.COLOR_BGR2GRAY)
        edges=cv2.Canny(gray,70,180)
        minimum=max(20,int(min(img.shape[:2])*.03))
        lines=cv2.HoughLinesP(edges,1,np.pi/180,threshold=65,minLineLength=minimum,maxLineGap=8)
        lines=[] if lines is None else [list(map(int,x[0])) for x in lines[:800]]
        preview=img.copy()
        for x1,y1,x2,y2 in lines:cv2.line(preview,(x1,y1),(x2,y2),(0,0,255),2)
        fn=f'candidates_page_{idx+1}.png';cv2.imwrite(str(out_dir/fn),preview)
        result['pages'].append({'page':idx+1,'width_px':img.shape[1], 'height_px':img.shape[0],
                                'candidate_lines_px':lines,'preview':fn,
                                'scale_calibrated':False,'verified':False})
    (out_dir/'scan_candidates.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf8')
    return len(imgs),sum(len(p['candidate_lines_px']) for p in result['pages'])

if __name__ == '__main__':
    p=argparse.ArgumentParser();p.add_argument('input',type=Path);p.add_argument('output',type=Path)
    a=p.parse_args();print('Pages and candidate lines:', analyse(a.input,a.output))
