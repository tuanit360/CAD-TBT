import base64,json
from pathlib import Path
from playwright.sync_api import sync_playwright
root=Path(__file__).resolve().parents[1];out=root/'EVIDENCE_R134';out.mkdir(exist_ok=True)
html=(root/'index.html').read_text();asset=base64.b64encode((root/'assets/ban_ve_mau.png').read_bytes()).decode();html=html.replace("data:'assets/ban_ve_mau.png'",f"data:'data:image/png;base64,{asset}'")
with sync_playwright() as pw:
 browser=pw.chromium.launch(executable_path='/usr/bin/chromium',args=['--no-sandbox','--disable-dev-shm-usage']);page=browser.new_page(viewport={'width':1680,'height':1050},accept_downloads=True)
 errors=[];page.on('pageerror',lambda e:errors.append(str(e)));page.set_content(html,wait_until='load');page.wait_for_function('image.complete && image.naturalWidth>0')
 def click(x,y):
  b=page.locator('#canvas').bounding_box();w,h=page.evaluate('[canvas.width,canvas.height]');page.mouse.click(b['x']+x*b['width']/w,b['y']+y*b['height']/h)
 page.locator('#calButton').click();page.locator('#realDistance').fill('10');click(100,100);click(300,100)
 page.locator('#length').click();click(100,230);click(300,230)
 page.locator('#selectMarkup').click();click(200,230)
 page.locator('#reviewStatus').select_option('da_xac_nhan');page.get_by_role('button',name='Lưu trạng thái').click()
 assert page.evaluate('measurements[0].status')=='da_xac_nhan'
 page.screenshot(path=str(out/'01_TRUOC_DOI_TY_LE_10M.png'),full_page=True)
 page.locator('#realDistance').fill('20');page.locator('#calButton').click();click(100,100);click(300,100)
 assert abs(page.evaluate('measurements[0].quantity')-20)<1e-7
 assert page.evaluate('measurements[0].status')=='chua_kiem_tra'
 assert page.evaluate("auditTrail.some(x=>x.action==='recalibrate_page')")
 page.screenshot(path=str(out/'02_SAU_DOI_TY_LE_20M.png'),full_page=True)
 page.get_by_role('button',name='↶ Hoàn tác').click();assert page.evaluate('measurements[0].quantity')==10
 page.get_by_role('button',name='↷ Làm lại').click();assert page.evaluate('measurements[0].quantity')==20
 with page.expect_download() as dl:page.get_by_role('button',name='Lưu dự án JSON').click()
 saved=out/'PROJECT_R134.json';dl.value.save_as(saved)
 page.locator('#projectFile').set_input_files(str(saved));page.wait_for_timeout(250)
 assert page.evaluate('measurements[0].quantity')==20
 assert not errors,errors
 result={'browser':'Playwright Chromium','outcome':'PASS','before_length_m':10,'after_length_m':20,'approval_revoked':True,'undo_redo':True,'save_reopen':True,'audit':True,'native_exe_tested':False,'errors':errors}
 (out/'RESULT.json').write_text(json.dumps(result,indent=2),encoding='utf-8');print(result);browser.close()
