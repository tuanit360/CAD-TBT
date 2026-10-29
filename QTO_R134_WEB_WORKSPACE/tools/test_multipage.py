from pathlib import Path
from playwright.sync_api import sync_playwright
from tempfile import TemporaryDirectory
import fitz
import subprocess,time,json,base64
root=Path(__file__).resolve().parents[1]
with TemporaryDirectory() as tmp:
    pdf=fitz.open()
    for i in range(2):
        p=pdf.new_page(width=600,height=400)
        p.draw_rect(fitz.Rect(60+i*10,60,350,310),color=(0,0,0),width=2)
    original=Path(tmp)/'two_pages.pdf';pdf.save(original);pdf.close()
    pages=Path(tmp)/'pages'
    subprocess.run(['python',str(root/'tools/render_pdf_pages.py'),str(original),'--out',str(pages)],check=True,stdout=subprocess.DEVNULL)
    server=subprocess.Popen(['python','-m','http.server','8774','--bind','127.0.0.1'],cwd=root,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    try:
        time.sleep(.4)
        with sync_playwright() as pw:
            browser=pw.chromium.launch(headless=True,executable_path='/usr/bin/chromium',args=['--no-sandbox'])
            page=browser.new_page(accept_downloads=True,viewport={'width':1600,'height':1000})
            errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
            html=(root/'index.html').read_text(encoding='utf-8').replace('assets/ban_ve_mau.png','data:image/png;base64,'+base64.b64encode((root/'assets/ban_ve_mau.png').read_bytes()).decode())
            page.set_content(html)
            page.locator('#fileImage').set_input_files([str(pages/'001.png'),str(pages/'002.png')])
            page.wait_for_timeout(500)
            assert page.locator('#pageIndicator').inner_text()=='Trang 1/2'
            def click(p,x,y):
                box=p.locator('#canvas').bounding_box();p.mouse.click(box['x']+x*box['width'],box['y']+y*box['height'])
            page.locator('#realDistance').fill('10');page.locator('#calButton').click()
            click(page,.2,.2);click(page,.4,.2)
            page.locator('#length').click();click(page,.2,.3);click(page,.4,.3)
            assert page.locator('#itemCount').inner_text()=='1'
            page.get_by_role('button',name='Trang sau').click()
            print('DEBUG CURRENT',page.locator('#pageIndicator').inner_text(),page.locator('#feedback').inner_text())
            assert page.locator('#pageIndicator').inner_text()=='Trang 2/2'
            assert 'Chưa hiệu chỉnh' in page.locator('#calStatus').inner_text()
            page.locator('#calButton').click();click(page,.2,.2);click(page,.4,.2)
            page.locator('#length').click();click(page,.2,.4);click(page,.4,.4)
            assert page.locator('#itemCount').inner_text()=='2'
            page.get_by_role('button',name='Hoàn tác').click();assert page.locator('#itemCount').inner_text()=='1'
            page.get_by_role('button',name='Làm lại').click();assert page.locator('#itemCount').inner_text()=='2'
            with page.expect_download() as event:page.get_by_role('button',name='Lưu dự án JSON').click()
            saved=Path(tmp)/'project.json';event.value.save_as(str(saved))
            data=json.loads(saved.read_text())
            assert len(data['pages'])==2 and {m['page'] for m in data['measurements']}=={1,2}
            print('DEBUG SAVED PAGE',data['pageIndex'])
            page.close();page=browser.new_page(accept_downloads=True,viewport={'width':1600,'height':1000});page.on('pageerror',lambda e:errors.append(str(e)));page.set_content(html);page.locator('#projectFile').set_input_files(str(saved));page.wait_for_timeout(500)
            print('DEBUG CURRENT',page.locator('#pageIndicator').inner_text(),page.locator('#feedback').inner_text())
            assert page.locator('#pageIndicator').inner_text()=='Trang 2/2'
            assert page.locator('#itemCount').inner_text()=='2'
            page.get_by_role('button',name='Trang trước').click()
            assert 'Đã hiệu chỉnh' in page.locator('#calStatus').inner_text()
            assert not errors,errors
            page.screenshot(path=str(root/'KIEM_THU_PDF_2_TRANG.png'))
            browser.close()
            for n in [1,2]:
                result=Path(tmp)/f'mark_{n}.pdf'
                subprocess.run(['python',str(root/'tools/pdf_tools.py'),'mark',str(original),str(saved),'--page',str(n),'--out',str(result)],check=True,stdout=subprocess.DEVNULL)
                with fitz.open(result) as marked:
                    assert len(list(marked[n-1].annots()))==1
                    assert not list(marked[(n%2)].annots())
            print('PASS MULTIPAGE_CHROMIUM_CLICK_UNDO_REDO_SAVE_REOPEN_PDF_MARK_2_PAGES')
    finally:server.terminate()
