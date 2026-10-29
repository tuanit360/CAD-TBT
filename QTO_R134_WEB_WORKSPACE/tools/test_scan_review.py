from pathlib import Path
from playwright.sync_api import sync_playwright
import json, base64
root=Path(__file__).resolve().parents[1]
with sync_playwright() as pw:
 browser=pw.chromium.launch(headless=True,executable_path='/usr/bin/chromium',args=['--no-sandbox'])
 page=browser.new_page(viewport={'width':1550,'height':1000})
 errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
 html=(root/'index.html').read_text().replace('assets/ban_ve_mau.png','data:image/png;base64,'+base64.b64encode((root/'assets/ban_ve_mau.png').read_bytes()).decode())
 page.set_content(html)
 page.locator('#fileImage').set_input_files(str(root/'assets/ban_ve_mau.png'))
 page.wait_for_timeout(300)
 page.locator('#scanCandidatesFile').set_input_files(str(root/'SCAN_CANDIDATES/scan_candidates.json'))
 page.wait_for_timeout(250)
 assert 'Ứng viên' in page.locator('#scanProgress').inner_text(),page.locator('#feedback').inner_text()
 page.get_by_role('button',name='Xác nhận nét đang chọn làm chiều dài').click()
 assert 'Cần hiệu chỉnh' in page.locator('#feedback').inner_text()
 page.evaluate('calibration={metresPerPixel:0.01,points:[[0,0],[100,0]]}; pages[pageIndex].calibration=calibration')
 page.get_by_role('button',name='Xác nhận nét đang chọn làm chiều dài').click()
 assert page.locator('#itemCount').inner_text()=='1'
 page.get_by_role('button',name='Xác nhận nét đang chọn làm chiều dài').click()
 assert 'trùng' in page.locator('#feedback').inner_text()
 page.get_by_role('button',name='Hoàn tác').click()
 assert page.locator('#itemCount').inner_text()=='0'
 page.get_by_role('button',name='Làm lại').click()
 assert page.locator('#itemCount').inner_text()=='1'
 assert not errors,errors
 page.screenshot(path=str(root/'KIEM_THU_SCAN_REVIEW.png'))
 print('PASS CHROMIUM_SCAN_CANDIDATES_REVIEW_SCALE_DEDUP_UNDO_REDO')
 browser.close()
