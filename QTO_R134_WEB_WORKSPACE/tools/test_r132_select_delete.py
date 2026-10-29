import base64,json
from pathlib import Path
from playwright.sync_api import sync_playwright
root=Path(__file__).resolve().parents[1]
out=root/'EVIDENCE_R132';out.mkdir(exist_ok=True)
html=(root/'index.html').read_text(encoding='utf8')
asset=base64.b64encode((root/'assets/ban_ve_mau.png').read_bytes()).decode()
html=html.replace("data:'assets/ban_ve_mau.png'", f"data:'data:image/png;base64,{asset}'")
with sync_playwright() as pw:
 browser=pw.chromium.launch(executable_path='/usr/bin/chromium',args=['--no-sandbox','--disable-dev-shm-usage'])
 page=browser.new_page(viewport={'width':1680,'height':1040},accept_downloads=True)
 errors=[];page.on('pageerror',lambda err:errors.append(str(err)))
 page.set_content(html,wait_until='load');page.wait_for_function('image.complete && image.naturalWidth > 0')
 def click(x,y):
  box=page.locator('#canvas').bounding_box();w,h=page.evaluate('[canvas.width,canvas.height]')
  page.mouse.click(box['x']+x*box['width']/w,box['y']+y*box['height']/h)
 page.locator('#calButton').click();click(100,100);click(300,100)
 page.locator('#length').click();click(100,200);click(300,200)
 page.locator('#area').click()
 for p in [(100,300),(300,300),(300,430),(100,430)]:click(*p)
 page.get_by_role('button',name='Kết thúc vùng').click()
 assert page.evaluate('measurements.length')==2
 page.locator('#selectMarkup').click()
 click(200,200)
 assert page.evaluate('measurements.find(x=>x.id===selected).kind')=='length'
 page.screenshot(path=str(out/'01_CHON_DAU_DO_TRUC_TIEP.png'),full_page=True)
 page.get_by_role('button',name='Xóa phép đo').click()
 assert page.evaluate('measurements.length')==1
 assert page.evaluate("auditTrail.some(a=>a.action==='delete_measurement')")
 page.screenshot(path=str(out/'02_SAU_XOA_DAU_DO_VA_GIU_VUNG.png'),full_page=True)
 page.get_by_role('button',name='↶ Hoàn tác').click()
 assert page.evaluate('measurements.length')==2
 page.get_by_role('button',name='↷ Làm lại').click()
 assert page.evaluate('measurements.length')==1
 click(200,350)
 assert page.evaluate('measurements.find(x=>x.id===selected).kind')=='area'
 with page.expect_download() as download:page.get_by_role('button',name='Lưu dự án JSON').click()
 download.value.save_as(out/'DU_AN_CHON_XOA.json')
 page.locator('#projectFile').set_input_files(str(out/'DU_AN_CHON_XOA.json'))
 page.wait_for_timeout(300)
 assert page.evaluate('measurements.length')==1
 assert page.evaluate('measurements[0].kind')=='area'
 assert not errors,errors
 report={'tested':'click segment to select; delete; audit; undo; redo; click area interior; save/reopen','measurements_before':2,'measurements_after':1,'browser':'Chromium via Playwright','status':'PASS','native_exe_verified':False}
 (out/'TEST_RESULT.json').write_text(json.dumps(report,indent=2,ensure_ascii=False))
 print(json.dumps(report,ensure_ascii=False))
 browser.close()
