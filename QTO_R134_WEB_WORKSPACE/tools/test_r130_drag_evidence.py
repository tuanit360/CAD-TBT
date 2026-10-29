import base64,json
from pathlib import Path
from playwright.sync_api import sync_playwright
root=Path(__file__).resolve().parents[1]
out=root/'EVIDENCE_R130';out.mkdir(exist_ok=True)
html=(root/'index.html').read_text(encoding='utf8')
image=base64.b64encode((root/'assets/ban_ve_mau.png').read_bytes()).decode()
html=html.replace("data:'assets/ban_ve_mau.png'",f"data:'data:image/png;base64,{image}'")
with sync_playwright() as p:
 b=p.chromium.launch(executable_path='/usr/bin/chromium',args=['--no-sandbox','--disable-dev-shm-usage'])
 page=b.new_page(viewport={'width':1640,'height':1000},accept_downloads=True)
 errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
 page.set_content(html,wait_until='load');page.wait_for_function('image.complete && image.naturalWidth > 0')
 def click(x,y):page.locator('#canvas').click(position={'x':x,'y':y})
 page.locator('#calButton').click();page.locator('#realDistance').fill('10');click(100,100);click(300,100)
 page.locator('#length').click();click(100,200);click(300,200)
 assert abs(page.evaluate('measurements[0].quantity')-10)<.01
 page.screenshot(path=str(out/'01_TRUOC_SUA_DIEM.png'),full_page=True)
 page.locator('#rows tr').first.click();page.locator('#reviewStatus').select_option('da_xac_nhan');page.get_by_role('button',name='Lưu trạng thái').click()
 assert page.evaluate('measurements[0].status')=='da_xac_nhan'
 page.locator('#editPoint').click()
 canvas=page.locator('#canvas');box=canvas.bounding_box();cw,ch=page.evaluate('[canvas.width,canvas.height]')
 def pos(x,y):return {'x':box['x']+x*box['width']/cw,'y':box['y']+y*box['height']/ch}
 points=page.evaluate('measurements[0].points'); start=pos(*points[1]); end=pos(points[1][0]+(points[1][0]-points[0][0])/2,points[1][1])
 page.mouse.move(start['x'],start['y']);page.mouse.down();page.mouse.move(end['x'],end['y'],steps=12);page.mouse.up()
 q=page.evaluate('measurements[0].quantity');status=page.evaluate('measurements[0].status')
 assert abs(q-15)<.03,(q,status)
 assert status=='chua_kiem_tra',status
 page.screenshot(path=str(out/'02_SAU_KEO_DIEM_CAN_DUYET_LAI.png'),full_page=True)
 page.get_by_role('button',name='↶ Hoàn tác').click()
 assert abs(page.evaluate('measurements[0].quantity')-10)<.01
 page.get_by_role('button',name='↷ Làm lại').click()
 assert abs(page.evaluate('measurements[0].quantity')-15)<.03
 with page.expect_download() as download:page.get_by_role('button',name='Lưu dự án JSON').click()
 download.value.save_as(out/'DU_AN_SAU_SUA_DIEM.json')
 saved=json.loads((out/'DU_AN_SAU_SUA_DIEM.json').read_text())
 assert abs(saved['measurements'][0]['quantity']-15)<.03
 page.locator('#projectFile').set_input_files(str(out/'DU_AN_SAU_SUA_DIEM.json'))
 page.wait_for_timeout(350)
 assert abs(page.evaluate('measurements[0].quantity')-15)<.03
 assert not errors,errors
 report={'test':'actual Chromium pointer drag on TBT360 workspace canvas','calibrated_meters_per_pixel':0.05,'before_length_m':10,'after_length_m':q,'approval_revoked_after_edit':status=='chua_kiem_tra','undo_redo':True,'save_reopen':True,'screenshots':['01_TRUOC_SUA_DIEM.png','02_SAU_KEO_DIEM_CAN_DUYET_LAI.png'],'native_rust_exe_verified':False}
 (out/'R130_BROWSER_TEST.json').write_text(json.dumps(report,ensure_ascii=False,indent=2))
 print(json.dumps(report,ensure_ascii=False));b.close()
