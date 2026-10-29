import base64,json
from pathlib import Path
from playwright.sync_api import sync_playwright
root=Path(__file__).resolve().parents[1];out=root/'EVIDENCE_R131';out.mkdir(exist_ok=True)
html=(root/'index.html').read_text(encoding='utf8'); image=base64.b64encode((root/'assets/ban_ve_mau.png').read_bytes()).decode();html=html.replace("data:'assets/ban_ve_mau.png'",f"data:'data:image/png;base64,{image}'")
with sync_playwright() as pw:
 browser=pw.chromium.launch(executable_path='/usr/bin/chromium',args=['--no-sandbox','--disable-dev-shm-usage'])
 page=browser.new_page(viewport={'width':1650,'height':1020},accept_downloads=True); errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
 page.set_content(html,wait_until='load');page.wait_for_function('image.complete && image.naturalWidth > 0')
 def click(x,y):page.locator('#canvas').click(position={'x':x,'y':y})
 def drag(x1,y1,x2,y2):
  box=page.locator('#canvas').bounding_box();w,h=page.evaluate('[canvas.width,canvas.height]')
  a=(box['x']+x1*box['width']/w,box['y']+y1*box['height']/h);b=(box['x']+x2*box['width']/w,box['y']+y2*box['height']/h)
  page.mouse.move(*a);page.mouse.down();page.mouse.move(*b,steps=12);page.mouse.up()
 page.locator('#calButton').click();page.locator('#realDistance').fill('10');click(100,100);click(300,100)
 page.locator('#area').click()
 for pt in [(100,200),(300,200),(300,360),(100,360)]:click(*pt)
 page.get_by_role('button',name='Kết thúc vùng').click()
 q0=page.evaluate('measurements[0].quantity');assert abs(q0-80)<.01,q0
 page.locator('#rows tr').first.click();page.locator('#reviewStatus').select_option('da_xac_nhan');page.get_by_role('button',name='Lưu trạng thái').click()
 page.screenshot(path=str(out/'01_DIEN_TICH_TRUOC_SUA_80M2.png'),full_page=True)
 page.locator('#editPoint').click();point=page.evaluate('measurements[0].points[1]');drag(point[0],point[1],point[0]+100,point[1])
 q1=page.evaluate('measurements[0].quantity');status=page.evaluate('measurements[0].status')
 assert q1>q0+5,(q1,status);assert status=='chua_kiem_tra'
 page.screenshot(path=str(out/'02_SAU_KEO_GRIP_DIEN_TICH.png'),full_page=True)
 page.get_by_role('button',name='↶ Hoàn tác').click();assert abs(page.evaluate('measurements[0].quantity')-80)<.01
 page.get_by_role('button',name='↷ Làm lại').click();assert abs(page.evaluate('measurements[0].quantity')-q1)<.01
 with page.expect_download() as download:page.get_by_role('button',name='Lưu dự án JSON').click()
 download.value.save_as(out/'DU_AN_SUA_VUNG.json')
 page.locator('#projectFile').set_input_files(str(out/'DU_AN_SUA_VUNG.json'));page.wait_for_timeout(400)
 assert abs(page.evaluate('measurements[0].quantity')-q1)<.01
 # invalid self intersection must roll back
 page.locator('#editPoint').click();point=page.evaluate('measurements[0].points[1]');drag(point[0],point[1],150,400)
 assert abs(page.evaluate('measurements[0].quantity')-q1)<.01,'Invalid polygon should roll back'
 assert not errors, errors
 report={'test':'real pointer grip polygon edit in TBT360 browser workspace','area_before_m2':q0,'area_after_m2':q1,'review_revoked':status=='chua_kiem_tra','undo_redo':True,'save_reopen':True,'invalid_polygon_rollback':True,'native_rust_verified':False}
 (out/'R131_BROWSER_TEST.json').write_text(json.dumps(report,ensure_ascii=False,indent=2));print(json.dumps(report,ensure_ascii=False));browser.close()
