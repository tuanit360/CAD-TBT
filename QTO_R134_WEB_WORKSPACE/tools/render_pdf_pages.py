#!/usr/bin/env python3
"""Render all PDF pages for the offline QTO workspace. Pages preserve order."""
import argparse
from pathlib import Path
import fitz

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('pdf',type=Path)
    parser.add_argument('--out',type=Path,default=Path('PAGES'))
    parser.add_argument('--dpi',type=int,default=144)
    args=parser.parse_args()
    if args.dpi<72 or args.dpi>300:
        parser.error('DPI must be 72–300')
    args.out.mkdir(parents=True,exist_ok=True)
    with fitz.open(args.pdf) as document:
        if len(document)>100:
            parser.error('More than 100 pages not supported')
        for index,page in enumerate(document):
            target=args.out/f'{index+1:03d}.png'
            page.get_pixmap(matrix=fitz.Matrix(args.dpi/72,args.dpi/72),alpha=False).save(target)
            print(f'{index+1}/{len(document)}: {target}')
    print('Open index.html, then select all PNG files with the multi-page file picker.')

if __name__=='__main__':main()
