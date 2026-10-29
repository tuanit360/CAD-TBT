"""Real Chromium click regression for independent PDF and 3D quantity review."""
from pathlib import Path
import base64, json, tempfile
from playwright.sync_api import sync_playwright
root=Path(__file__).resolve().parents[1]
uri='data:image/png;base64,'+base64.b64encode((root/'assets/ban_ve_mau.png').read_bytes()).decode()
with sync_playwright() as pw:
 browser=pw.chromium.launch(headless=True,executable_path='/usr/bin/chromium',args=['--no-sandbox'])
 page=browser.new_page(viewport={'width':1500,'height':950},accept_downloads=True)
 errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
 page.set_content((root/'index.html').read_text().replace('assets/ban_ve_mau.png',uri))
 page.evaluate('''src=>{pages=[{name:'Sheet 01',data:src,calibration:{metresPerPixel:.01,points:[[0,0],[100,0]]}}];pageIndex=0;loadPage(true);
 measurements=[{id:'PDF-1',source:'PDF',sourceFile:'sheet01.pdf',page:1,classification:'SÀN',description:'Diện tích 2D',kind:'area',quantity:80,unit:'m²',status:'chua_kiem_tra',points:[[20,20],[100,20],[100,100]]}];
 modelRows=[{id:'SOLID-1',source:'STEP_VOLUME_MAP',sourceFile:'solids.csv',classification:'SÀN',description:'Solid',kind:'volume',quantity:12,unit:'m³',status:'chua_kiem_tra'}];render();}''',uri)
 page.wait_for_timeout(150)
 page.locator('#rows tr').first.click()
 page.locator('#editGroup').fill('BETONG_SAN')
 page.locator('#editDescription').fill('Sàn tầng 1 - duyệt bản vẽ')
 page.locator('#editNote').fill('Đối chiếu bản vẽ số A101')
 page.get_by_role('button',name='Lưu diễn giải và nhật ký').click()
 assert page.evaluate('measurements[0].classification')=='BETONG_SAN'
 assert page.evaluate('auditTrail.length')==1
 page.evaluate('undo()')
 assert page.evaluate('measurements[0].classification')=='SÀN'
 assert page.evaluate('auditTrail.length')==0
 page.evaluate('redo()')
 assert page.evaluate('auditTrail.length')==1
 page.locator('#reviewStatus').select_option('da_xac_nhan')
 page.get_by_role('button',name='Lưu trạng thái').click()
 assert page.evaluate('measurements[0].status')=='da_xac_nhan'
 page.get_by_role('button',name='Mô hình 3D / QTO').click()
 page.locator('#rows tr').first.click()
 page.locator('#editNote').fill('Đối chiếu Solid BREP R117')
 page.get_by_role('button',name='Lưu diễn giải và nhật ký').click()
 page.locator('#reviewStatus').select_option('da_xac_nhan')
 page.get_by_role('button',name='Lưu trạng thái').click()
 result=page.evaluate('approvedInventory()')
 assert len(result['rows'])==2, result
 assert {r['unit'] for r in result['rows']}=={'m²','m³'}
 with page.expect_download() as info: page.get_by_role('button',name='Xuất nhật ký CSV').click()
 assert info.value.suggested_filename=='TBT360_QTO_NHAT_KY.csv'
 with page.expect_download() as info: page.evaluate('saveProject()')
 path=Path(tempfile.mkdtemp())/'qto-test.json';info.value.save_as(path)
 data=json.loads(path.read_text());assert len(data['auditTrail'])==4
 page.locator('#projectFile').set_input_files(str(path));page.wait_for_timeout(200)
 assert page.evaluate('auditTrail.length')==4
 assert page.evaluate('measurements[0].reviewNote')=='Đối chiếu bản vẽ số A101'
 assert not errors,errors
 browser.close()
 print('PASS chromium: actual markup selection/edit; revision journal; undo/redo; PDF 2D + 3D quantities separated; export journal; save/reopen')
