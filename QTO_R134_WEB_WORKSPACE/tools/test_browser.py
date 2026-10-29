from pathlib import Path
import subprocess,time
from playwright.sync_api import sync_playwright
p=Path(__file__).resolve().parents[1]/'index.html'
server=subprocess.Popen(['python','-m','http.server','8765','--bind','127.0.0.1'],cwd=str(p.parent),stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
time.sleep(.4)
try:
 with sync_playwright() as pw:
    browser=pw.chromium.launch(headless=True,executable_path='/usr/bin/chromium',args=['--no-sandbox'])
    page=browser.new_page(accept_downloads=True,viewport={'width':1530,'height':950})
    errs=[]
    page.on('pageerror',lambda e:errs.append(str(e)))
    page.goto('http://127.0.0.1:8765/index.html')
    page.wait_for_timeout(600)
    assert page.locator('#canvas').evaluate('(c)=>c.width')>1000
    page.locator('#realDistance').fill('10')
    page.locator('#calButton').click()
    box=page.locator('#canvas').bounding_box()
    def click(x,y):page.mouse.click(box['x']+x*box['width'],box['y']+y*box['height'])
    click(.1,.1);click(.3,.1)
    assert page.locator('#calStatus').inner_text().startswith('Đã hiệu chỉnh')
    page.locator('#length').click();click(.15,.2);click(.35,.2)
    assert page.locator('#itemCount').inner_text()=='1'
    page.get_by_role('button',name='Hoàn tác').click();assert page.locator('#itemCount').inner_text()=='0'
    page.get_by_role('button',name='Làm lại').click();assert page.locator('#itemCount').inner_text()=='1'
    page.locator('#area').click();click(.1,.4);click(.3,.4);click(.3,.6);click(.1,.6)
    page.get_by_role('button',name='Kết thúc vùng').click();assert page.locator('#itemCount').inner_text()=='2'
    page.locator('#mode3d').click()
    volume=Path(__file__).resolve().parents[1]/'assets'/'OBJECT_VOLUME_MAP.csv'
    page.locator('#volumeFile').set_input_files(str(volume))
    assert int(page.locator('#itemCount').inner_text())>45
    with page.expect_download() as d:page.get_by_role('button',name='Lưu dự án JSON').click()
    save=d.value
    assert save.suggested_filename.endswith('.json')
    with page.expect_download() as d2:page.get_by_role('button',name='Chi tiết CSV').click()
    assert d2.value.suggested_filename.endswith('.csv')
    page.screenshot(path=str(Path(__file__).resolve().parents[1]/'GIAO_DIEN_DA_TEST.png'),full_page=True)
    print('PASS_BROWSER_CALIBRATE_MEASURE_UNDO_REDO_AREA_IMPORT_3D_SAVE_CSV')
    print('js_errors',errs)
    assert not errs
    browser.close()

finally:
 server.terminate()
