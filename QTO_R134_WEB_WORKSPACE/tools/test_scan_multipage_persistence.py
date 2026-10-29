"""Browser regression for scan candidate isolation and project reopen."""
from pathlib import Path
import base64
import json
from playwright.sync_api import sync_playwright
root=Path(__file__).resolve().parents[1]
image=root/'assets/ban_ve_mau.png'
url='data:image/png;base64,'+base64.b64encode(image.read_bytes()).decode()
with sync_playwright() as pw:
    browser=pw.chromium.launch(headless=True, executable_path='/usr/bin/chromium', args=['--no-sandbox'])
    page=browser.new_page(viewport={'width':1550,'height':960},accept_downloads=True)
    errors=[]
    page.on('pageerror',lambda e:errors.append(str(e)))
    html=(root/'index.html').read_text().replace('assets/ban_ve_mau.png',url)
    page.set_content(html)
    page.evaluate('''(src) => { pages=[{name:'Trang 1',data:src,calibration:{metresPerPixel:0.01,points:[[0,0],[100,0]]}},{name:'Trang 2',data:src,calibration:{metresPerPixel:0.02,points:[[0,0],[100,0]]}}];pageIndex=0;loadPage(true); }''',url)
    page.wait_for_timeout(200)
    page.evaluate("scanPageStore={'1':{source:'test',lines:[[[10,10],[110,10]]]},'2':{source:'test',lines:[[[20,20],[20,70]]]}};restoreScanPage();render();")
    assert page.evaluate('scanCandidates.length')==1
    page.evaluate('acceptScanLine()')
    assert page.evaluate('measurements.length')==1
    page.evaluate('changePage(1)')
    assert page.evaluate('scanCandidates[0][0][0]')==20
    page.evaluate('acceptScanLine()')
    assert page.evaluate('measurements.length')==2
    assert page.evaluate('measurements[1].page')==2
    page.evaluate('changePage(-1)')
    assert page.evaluate('scanCandidates[0][0][0]')==10
    page.evaluate('undo()')
    assert page.evaluate('measurements.length')==1
    page.evaluate('redo()')
    assert page.evaluate('measurements.length')==2
    with page.expect_download() as info: page.evaluate('saveProject()')
    downloaded=info.value
    save=root/'TEST_R125_PROJECT.json';downloaded.save_as(save)
    payload=json.loads(save.read_text())
    assert len(payload['scanPageStore'])==2
    assert len(payload['measurements'])==2
    page.locator('#projectFile').set_input_files(str(save))
    page.wait_for_timeout(450)
    assert page.evaluate('Object.keys(scanPageStore).length')==2
    assert page.evaluate('measurements.length')==2
    page.evaluate('changePage(1)')
    assert page.evaluate('scanCandidates[0][0][0]')==20
    assert not errors,errors
    print('PASS: page-isolated candidates; measures; undo/redo; JSON Save/Reopen; correct candidate restoration')
    browser.close()
