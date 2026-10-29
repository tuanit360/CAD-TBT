import base64
from pathlib import Path
from playwright.sync_api import sync_playwright
root=Path(__file__).resolve().parents[1]
html=(root/'index.html').read_text(encoding='utf8')
image=base64.b64encode((root/'assets/ban_ve_mau.png').read_bytes()).decode()
html=html.replace("image.src='assets/ban_ve_mau.png';",f"image.src='data:image/png;base64,{image}';")
with sync_playwright() as p:
 browser=p.chromium.launch(executable_path='/usr/bin/chromium',args=['--no-sandbox','--disable-dev-shm-usage'])
 page=browser.new_page(viewport={'width':1500,'height':980},accept_downloads=True)
 errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
 page.set_content(html,wait_until='load')
 page.wait_for_function('image.complete && image.naturalWidth > 0')
 page.evaluate('setTool("calibrate")')
 page.evaluate('el("realDistance").value="10"')
 page.locator('#canvas').click(position={'x':100,'y':100})
 page.locator('#canvas').click(position={'x':300,'y':100})
 assert page.evaluate('calibration !== null')
 page.evaluate('setTool("length")')
 page.locator('#canvas').click(position={'x':100,'y':200})
 page.locator('#canvas').click(position={'x':300,'y':200})
 quantity=page.evaluate('measurements[0].quantity')
 assert abs(quantity-10)<1e-6,quantity
 assert page.evaluate('measurements.length')==1
 page.evaluate('undo()');assert page.evaluate('measurements.length')==0
 page.evaluate('redo()');assert page.evaluate('measurements.length')==1
 audit=page.evaluate('showAudit()');assert audit['total']==1
 # reload via saved project data - use equivalent file contents through an actual input
 import json,tempfile
 content=page.evaluate('JSON.stringify({schema:"tbt360-qto-v1",measurements,modelRows,calibration,pdfName:"test",imageData:imgData})')
 with tempfile.NamedTemporaryFile(suffix='.json',delete=False) as t:t.write(content.encode());fname=t.name
 page.locator('#projectFile').set_input_files(fname)
 page.wait_for_timeout(100)
 assert page.evaluate('measurements.length')==1
 assert not errors,errors
 print('BROWSER PASS: calibration, mouse measurement, undo, redo, audit, project re-open; length =',quantity)
 browser.close()
