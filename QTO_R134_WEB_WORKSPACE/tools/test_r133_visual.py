import base64, json
from pathlib import Path
from playwright.sync_api import sync_playwright
root=Path(__file__).resolve().parents[1]
out=root/'EVIDENCE_R133';out.mkdir(exist_ok=True)
html=(root/'index.html').read_text(encoding='utf8')
asset=base64.b64encode((root/'assets/ban_ve_mau.png').read_bytes()).decode()
html=html.replace("data:'assets/ban_ve_mau.png'", f"data:'data:image/png;base64,{asset}'")
with sync_playwright() as pw:
 browser=pw.chromium.launch(executable_path='/usr/bin/chromium',args=['--no-sandbox','--disable-dev-shm-usage'])
 page=browser.new_page(viewport={'width':1680,'height':1040},accept_downloads=True)
 errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
 page.set_content(html,wait_until='load');page.wait_for_function('image.complete && image.naturalWidth>0')
 def click(x,y):
  box=page.locator('#canvas').bounding_box();w,h=page.evaluate('[canvas.width,canvas.height]')
  page.mouse.click(box['x']+x*box['width']/w,box['y']+y*box['height']/h)
 page.locator('#calButton').click();page.locator('#realDistance').fill('10');click(100,100);click(300,100)
 page.locator('#length').click();click(100,240);click(300,240)
 page.locator('#selectMarkup').click();click(200,240)
 assert page.evaluate('selected===measurements[0].id')
 page.locator('#reviewStatus').select_option('da_xac_nhan');page.get_by_role('button',name='Lưu trạng thái').click()
 before=page.evaluate('structuredClone(measurements[0])')
 assert before['status']=='da_xac_nhan'
 page.screenshot(path=str(out/'01_TRUOC_DICH_CHUYEN_10M.png'),full_page=True)
 page.locator('#moveMarkup').click();click(100,240);click(150,310)
 after=page.evaluate('structuredClone(measurements[0])')
 assert abs(after['quantity']-before['quantity'])<1e-9,(before,after)
 assert all(abs(p[0]-q[0])<2 and abs(p[1]-q[1])<2 for p,q in zip(after['points'],[[150,310],[350,310]])),after
 assert after['status']=='chua_kiem_tra'
 assert page.evaluate("auditTrail.some(e=>e.action==='move_markup')")
 page.screenshot(path=str(out/'02_SAU_DICH_CHUYEN_GIU_10M.png'),full_page=True)
 page.get_by_role('button',name='↶ Hoàn tác').click()
 assert page.evaluate('measurements[0].points')==before['points']
 page.get_by_role('button',name='↷ Làm lại').click()
 assert page.evaluate('measurements[0].points')==after['points']
 with page.expect_download() as di:page.get_by_role('button',name='Lưu dự án JSON').click()
 save=out/'DU_AN_SAU_DI_CHUYEN.json';di.value.save_as(save)
 page.locator('#projectFile').set_input_files(str(save));page.wait_for_timeout(200)
 assert page.evaluate('measurements[0].points')==after['points']
 assert page.evaluate('measurements[0].quantity')==10
 assert not errors,errors
 report={'browser':'Chromium Playwright','native_rust_tested':False,'tests':{'real_canvas_button':True,'real_mouse_translation':True,'measurement_value_invariant':True,'approval_revoked_on_edit':True,'audit':True,'undo_redo':True,'save_reopen_json':True},'before':before,'after':after,'errors':errors}
 (out/'QA_R133.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf8')
 print(json.dumps(report['tests'],ensure_ascii=False))
 browser.close()
