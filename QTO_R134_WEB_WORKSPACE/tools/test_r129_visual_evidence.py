import base64,json
from pathlib import Path
from playwright.sync_api import sync_playwright
root=Path(__file__).resolve().parents[1]
evidence=root/'EVIDENCE_R129'; evidence.mkdir(exist_ok=True)
html=(root/'index.html').read_text(encoding='utf-8')
image=base64.b64encode((root/'assets/ban_ve_mau.png').read_bytes()).decode()
html=html.replace("data:'assets/ban_ve_mau.png'",f"data:'data:image/png;base64,{image}'")
with sync_playwright() as p:
 browser=p.chromium.launch(executable_path='/usr/bin/chromium',args=['--no-sandbox','--disable-dev-shm-usage'])
 page=browser.new_page(viewport={'width':1580,'height':1050}, device_scale_factor=1,accept_downloads=True)
 errors=[]; page.on('pageerror',lambda e:errors.append(str(e)))
 page.set_content(html,wait_until='load'); page.wait_for_function('image.complete && image.naturalWidth > 0')
 page.locator('#calButton').click(); page.locator('#realDistance').fill('10')
 page.locator('#canvas').click(position={'x':100,'y':100});page.locator('#canvas').click(position={'x':300,'y':100})
 assert page.evaluate('calibration !== null')
 page.locator('#length').click();page.locator('#canvas').click(position={'x':100,'y':200});page.locator('#canvas').click(position={'x':300,'y':200})
 page.locator('#area').click()
 for xy in [(400,220),(600,220),(600,370),(400,370)]:page.locator('#canvas').click(position={'x':xy[0],'y':xy[1]})
 page.get_by_role('button',name='Kết thúc vùng').click()
 page.locator('#count').click();page.locator('#canvas').click(position={'x':700,'y':400})
 values=page.evaluate('measurements.map(m=>({kind:m.kind,quantity:m.quantity,unit:m.unit,status:m.status}))')
 assert len(values)==3,values
 assert abs(values[0]['quantity']-10)<1e-8,values
 assert abs(values[1]['quantity']-75)<0.2,values
 assert values[2]['quantity']==1
 page.screenshot(path=str(evidence/'01_DO_TRUC_TIEP_TREN_WORKSPACE.png'),full_page=True)
 page.get_by_role('button',name='↶ Hoàn tác').click();assert page.evaluate('measurements.length')==2
 page.get_by_role('button',name='↷ Làm lại').click();assert page.evaluate('measurements.length')==3
 page.locator('#rows tr').first.click();page.locator('#reviewStatus').select_option('da_xac_nhan');page.get_by_role('button',name='Lưu trạng thái').click()
 page.screenshot(path=str(evidence/'02_DUYET_DAU_DO_VA_BANG_QTO.png'),full_page=True)
 downloads=[]
 page.on('download',lambda item:downloads.append(item))
 page.locator('#evidenceExport').click()
 page.wait_for_timeout(1000)
 assert len(downloads)>=2,[d.suggested_filename for d in downloads]
 for item in downloads:
  if item.suggested_filename.endswith('.json'):
   item.save_as(evidence/'BANG_CHUNG_TRANG_1.json')
  elif item.suggested_filename.endswith('.png'):
   item.save_as(evidence/'BAN_VE_DANH_DAU_TRUC_TIEP.png')
 evidence_doc=json.loads((evidence/'BANG_CHUNG_TRANG_1.json').read_text(encoding='utf-8'))
 assert len(evidence_doc['measurements'])==3
 assert evidence_doc['calibration']['metresPerPixel']>0
 page.wait_for_timeout(400)
 with page.expect_download() as d:page.get_by_role('button',name='Lưu dự án JSON').click()
 download=d.value; saved=evidence/'DU_AN_DO_BOC_DA_LUU.json';download.save_as(saved)
 doc=json.loads(saved.read_text(encoding='utf-8'));assert len(doc['measurements'])==3
 page.locator('#projectFile').set_input_files(str(saved));page.wait_for_timeout(200)
 assert page.evaluate('measurements.length')==3
 assert page.evaluate('measurements.filter(m=>m.status==="da_xac_nhan").length')==1
 assert not errors,errors
 report={'browser':'Chromium / Playwright actual pointer clicks','source':'R128 workspace, R129 visual acceptance','tests':{'real_calibration':True,'mouse_length_10m':True,'mouse_area_75m2':True,'mouse_count_1':True,'undo_redo':True,'review_approved':True,'download_save_reopen':True},'measures':values,'screenshot_files':['01_DO_TRUC_TIEP_TREN_WORKSPACE.png','02_DUYET_DAU_DO_VA_BANG_QTO.png'],'native_rust_exe_tested':False}
 (evidence/'R129_ACTUAL_UI_QA.json').write_text(json.dumps(report,indent=2,ensure_ascii=False),encoding='utf-8')
 print(json.dumps(report,ensure_ascii=False))
 browser.close()
