
'use strict';const el=id=>document.getElementById(id);const canvas=el('canvas'),ctx=canvas.getContext('2d');const viewport=el('viewport');let image=new Image(),imgData='',tool='',mode='pdf',draft=[],measurements=[],modelRows=[],history=[],future=[],calibration=null,calDraft=[],selected=null,zoom=1,pointerScale=1;const MAX_ROWS=20000;let pdfName='ban_ve_mau.pdf';let pages=[{name:'ban_ve_mau.pdf',data:'assets/ban_ve_mau.png',calibration:null}],pageIndex=0;
const toast=(s,bad=false)=>{el('feedback').textContent=s;el('feedback').className=bad?'status bad':'status good'};const finite=x=>Number.isFinite(x);function snapshot(){history.push(JSON.stringify({measurements,calibration,modelRows,pages,pageIndex,scanPageStore,auditTrail,comparisonBaseline}));if(history.length>100)history.shift();future=[]}function undo(){if(!history.length)return;future.push(JSON.stringify({measurements,calibration,modelRows,pages,pageIndex,scanPageStore,auditTrail,comparisonBaseline}));const o=JSON.parse(history.pop());({measurements,calibration,modelRows,pages,pageIndex,scanPageStore,auditTrail,comparisonBaseline}=o);restoreScanPage();loadPage(false)}function redo(){if(!future.length)return;history.push(JSON.stringify({measurements,calibration,modelRows,pages,pageIndex,scanPageStore,auditTrail,comparisonBaseline}));const o=JSON.parse(future.pop());({measurements,calibration,modelRows,pages,pageIndex,scanPageStore,auditTrail,comparisonBaseline}=o);restoreScanPage();loadPage(false)}
function changePage(delta){const n=pageIndex+delta;if(n<0||n>=pages.length)return;pages[pageIndex].calibration=calibration;pageIndex=n;restoreScanPage();loadPage(true)}function loadPage(fit){const pg=pages[pageIndex];calibration=pg.calibration||null;pdfName=pg.name;imgData=pg.data;draft=[];calDraft=[];selected=null;el('pageIndicator').textContent='Trang '+(pageIndex+1)+'/'+pages.length;image.onload=()=>{render();if(fit)fitCanvas()};image.src=pg.data;render()}function switchMode(m){mode=m;el('mode2d').classList.toggle('active',m==='pdf');el('mode3d').classList.toggle('active',m==='model');el('canvasTitle').textContent=m==='pdf'?'Bản vẽ / PDF · đo và đánh dấu':'Mô hình 3D · bảng cấu kiện tham chiếu';el('modelArea').style.background=m==='model'?'#eaf2fe':'white';renderRows()}
function setTool(t){tool=t;draft=[];calDraft=[];moveOrigin=null;['length','area','perimeter','count','calButton','editPoint','selectMarkup','moveMarkup'].forEach(id=>el(id).classList.remove('active'));el(t==='calibrate'?'calButton':t==='edit'?'editPoint':t==='select'?'selectMarkup':t==='move'?'moveMarkup':t).classList.add('active');el('toolBadge').textContent=({length:'Đo chiều dài',area:'Đo diện tích',perimeter:'Đo chu vi',count:'Đếm đối tượng',calibrate:'Chọn 2 điểm hiệu chỉnh',edit:'Sửa điểm đo (kéo đỉnh)',select:'Chọn dấu đo trực tiếp',move:'Di chuyển dấu đo (chọn 2 điểm)'})[t]||t;render()}function resetCalibration(){
 if(measurements.some(row=>(row.page||1)===pageIndex+1)){
  toast('Không được xóa tỷ lệ khi trang còn phép đo. Hãy hiệu chỉnh lại bằng 2 điểm chuẩn.',true);
  return;
 }
 snapshot();calibration=null;pages[pageIndex].calibration=null;render();
}
// An atomic, page-scoped revision. 2D/3D inventories never change on PDF calibration.
function applyPageCalibration(points,metres){
 const pixels=dist(...points);
 if(!finite(metres)||metres<=0||!finite(pixels)||pixels<2)throw Error('Khoảng chuẩn không hợp lệ');
 const scale=metres/pixels;
 const affected=measurements.filter(row=>(row.page||1)===pageIndex+1);
 const updates=affected.map(row=>{
  const [quantity,unit]=valueFor(row.kind,row.points,scale);
  if(!finite(quantity)||quantity<0||unit!==row.unit)throw Error('Không thể tính lại phép đo '+row.id);
  return {row,quantity,scale};
 });
 snapshot();
 const before=calibration;
 calibration={points:points.map(p=>p.slice()),metresPerPixel:scale,realMetres:metres};
 pages[pageIndex].calibration=calibration;
 for(const update of updates){
  const previous={quantity:update.row.quantity,scale:update.row.scale,status:update.row.status};
  update.row.quantity=update.quantity;
  update.row.scale=update.scale;
  if(update.row.status==='da_xac_nhan')update.row.status='chua_kiem_tra';
  recordTakeoffRevision(update.row,'recalibrate_page',previous,{quantity:update.row.quantity,scale:update.row.scale,status:update.row.status});
 }
 toast('Đã hiệu chỉnh trang '+(pageIndex+1)+'; tính lại '+updates.length+' phép đo và yêu cầu duyệt lại.');
 return {before,scale,affected:updates.length};
}
function getPos(e){const r=canvas.getBoundingClientRect();return [(e.clientX-r.left)*canvas.width/r.width,(e.clientY-r.top)*canvas.height/r.height]}
function dist(a,b){return Math.hypot(a[0]-b[0],a[1]-b[1])}function length(poly,closed){let s=0;for(let i=1;i<poly.length;i++)s+=dist(poly[i-1],poly[i]);if(closed&&poly.length>2)s+=dist(poly[poly.length-1],poly[0]);return s}function area(poly){let x=0;for(let i=0;i<poly.length;i++){const a=poly[i],b=poly[(i+1)%poly.length];x+=a[0]*b[1]-b[0]*a[1]}return Math.abs(x)/2}function polygonIsSimple(points){
 if(points.length<3)return false;
 const cross=(a,b,c)=>(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0]);
 const onSegment=(a,b,p)=>Math.min(a[0],b[0])-1e-8<=p[0]&&p[0]<=Math.max(a[0],b[0])+1e-8&&Math.min(a[1],b[1])-1e-8<=p[1]&&p[1]<=Math.max(a[1],b[1])+1e-8;
 for(let i=0;i<points.length;i++){
  const a=points[i],b=points[(i+1)%points.length];if(dist(a,b)<1e-5)return false;
  for(let j=i+1;j<points.length;j++){
   if(j===i+1||(i===0&&j===points.length-1))continue;
   const c=points[j],d=points[(j+1)%points.length];
   const x=cross(a,b,c),y=cross(a,b,d),z=cross(c,d,a),w=cross(c,d,b);
   if((x*y<0&&z*w<0)||(Math.abs(x)<1e-8&&onSegment(a,b,c))||(Math.abs(y)<1e-8&&onSegment(a,b,d))||(Math.abs(z)<1e-8&&onSegment(c,d,a))||(Math.abs(w)<1e-8&&onSegment(c,d,b)))return false;
  }
 }
 return area(points)>1e-4;
}
function valueFor(kind,pts,scale){if(kind==='count')return [1,'cái'];if(!scale||!finite(scale)||scale<=0)throw Error('Phải hiệu chỉnh tỷ lệ trước khi đo');if(kind==='area')return [area(pts)*scale*scale,'m²'];return [length(pts,kind==='perimeter')*scale,'m']}
// R132: hit-test the REAL canvas, not an image overlay. Click a segment or inside a polygon.
function segmentDistance(p,a,b){
 const dx=b[0]-a[0],dy=b[1]-a[1],den=dx*dx+dy*dy;
 const t=den>0?Math.max(0,Math.min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/den)):0;
 return Math.hypot(p[0]-(a[0]+t*dx),p[1]-(a[1]+t*dy));
}
function insidePolygon(p,points){
 let inside=false;
 for(let i=0,j=points.length-1;i<points.length;j=i++){
  const a=points[i],b=points[j];
  if((a[1]>p[1])!==(b[1]>p[1])&&p[0]<(b[0]-a[0])*(p[1]-a[1])/(b[1]-a[1])+a[0])inside=!inside;
 }
 return inside;
}
function selectMarkupAt(p){
 const candidates=measurements.filter(r=>(r.page||1)===pageIndex+1);
 let best=null;
 for(let index=candidates.length-1;index>=0;index--){
  const r=candidates[index];if(!Array.isArray(r.points)||!r.points.length)continue;
  let distance=Math.min(...r.points.map(q=>dist(p,q)));
  for(let i=1;i<r.points.length;i++)distance=Math.min(distance,segmentDistance(p,r.points[i-1],r.points[i]));
  if(r.points.length>2&&['area','perimeter'].includes(r.kind)){
   distance=Math.min(distance,segmentDistance(p,r.points[r.points.length-1],r.points[0]));
   if(insidePolygon(p,r.points))distance=0;
  }
  if(distance<=14&&(!best||distance<best.distance))best={row:r,distance};
 }
 if(!best){selected=null;render();toast('Không có dấu đo tại vị trí chọn');return;}
 const r=best.row;selected=r.id;
 el('reviewStatus').value=r.status;el('editGroup').value=r.classification||'';
 el('editDescription').value=r.description||'';el('editNote').value=r.reviewNote||'';
 el('canonicalLink').value=canonicalKey(r);render();
 toast('Đã chọn dấu đo '+r.id+'; có thể sửa, duyệt hoặc xóa');
}
canvas.addEventListener('click',e=>{const p=getPos(e);if(suppressClickAfterDrag){suppressClickAfterDrag=false;return;}if(tool==='edit')return;if(tool==='select'){selectMarkupAt(p);return;}if(tool==='move'){moveMarkupAt(p);return;}if(tool==='calibrate'){calDraft.push(p);if(calDraft.length===2){const met=Number(el('realDistance').value),px=dist(...calDraft);if(!finite(met)||met<=0||px<2){calDraft=[];toast('Khoảng chuẩn không hợp lệ',true);render();return}try{applyPageCalibration(calDraft,met)}catch(err){toast(err.message,true)}calDraft=[];tool=''}render();return}if(!['length','area','perimeter','count'].includes(tool))return;if(!calibration){toast('Hãy chọn 2 điểm chuẩn và nhập chiều dài thực tế trước',true);return}if(tool==='count'){commitMeasure([p]);return}draft.push(p);if(tool==='length'&&draft.length===2)commitMeasure(draft.splice(0));render()});
// R130: native canvas pointer manipulation of an existing markup, never an illustration.
let editDrag=null;let suppressClickAfterDrag=false;
canvas.addEventListener('pointerdown',e=>{
 if(tool!=='edit'||mode!=='pdf')return;
 const p=getPos(e), visible=measurements.filter(r=>(r.page||1)===pageIndex+1);
 let nearest=null;
 for(const row of visible){for(let i=0;i<row.points.length;i++){
  const d=dist(p,row.points[i]);if(d<=12&&(!nearest||d<nearest.distance))nearest={row,index:i,distance:d};
 }}
 if(!nearest){toast('Chọn đúng một đỉnh dấu đo để kéo',true);return;}
 if(nearest.row.status==='bo_qua'){toast('Không sửa phép đo đã bỏ qua',true);return;}
 selected=nearest.row.id;editDrag={id:nearest.row.id,index:nearest.index,original:nearest.row.points[nearest.index].slice(),moved:false,snapshotted:false};
 canvas.setPointerCapture(e.pointerId);render();
});
canvas.addEventListener('pointermove',e=>{
 if(!editDrag)return;
 const p=getPos(e);const row=measurements.find(r=>r.id===editDrag.id);if(!row)return;
 if(!editDrag.snapshotted){snapshot();editDrag.snapshotted=true;}
 row.points[editDrag.index]=p;editDrag.moved=true;render();
});
canvas.addEventListener('pointerup',e=>{
 if(!editDrag)return;const edit=editDrag;editDrag=null;
 if(!edit.moved)return;
 const row=measurements.find(r=>r.id===edit.id);if(!row)return;
 const pts=row.points;
 try{
  if(row.kind!=='count'){
   if((row.kind==='area'||row.kind==='perimeter')&&!polygonIsSimple(pts))throw Error('Vùng đo tự cắt, suy biến hoặc có cạnh bằng 0');
   const result=valueFor(row.kind,pts,row.scale);
   if(!finite(result[0])||result[0]<=0)throw Error('Hình học sau sửa không hợp lệ');
   row.quantity=result[0];row.unit=result[1];
  }
  if(row.status==='da_xac_nhan')row.status='chua_kiem_tra';
  recordTakeoffRevision(row,'geometry_edit',{points:[edit.original],status:'previous'},
    {points:[pts[edit.index]],status:row.status,quantity:row.quantity});
  suppressClickAfterDrag=true;render();toast('Đã sửa điểm đo; cần duyệt lại kết quả '+row.quantity.toFixed(3)+' '+row.unit);
 }catch(err){undo();toast(err.message,true);}
});
canvas.addEventListener('pointercancel',()=>{if(editDrag&&editDrag.snapshotted)undo();editDrag=null;});

// R133: Two-click translation of a selected actual canvas measurement.
// Keeps geometric quantity invariant; previously approved entries require re-review.
let moveOrigin=null;
function moveMarkupAt(point){
 const row=measurements.find(item=>item.id===selected&&(item.page||1)===pageIndex+1);
 if(!row){toast('Chọn một dấu đo trên trang hiện tại trước khi di chuyển',true);return;}
 if(!moveOrigin){moveOrigin=point;toast('Chọn điểm đích để di chuyển dấu đo '+row.id);return;}
 const dx=point[0]-moveOrigin[0],dy=point[1]-moveOrigin[1];moveOrigin=null;
 if(!Number.isFinite(dx)||!Number.isFinite(dy)||Math.hypot(dx,dy)<0.01){toast('Không có dịch chuyển',true);return;}
 const previous=row.points.map(p=>p.slice());const previousStatus=row.status;
 const moved=previous.map(p=>[p[0]+dx,p[1]+dy]);
 if(moved.some(p=>p.some(v=>!finite(v)))||moved.some(p=>p[0]<0||p[1]<0||p[0]>canvas.width||p[1]>canvas.height)){
  toast('Vị trí mới nằm ngoài trang bản vẽ',true);return;
 }
 snapshot();row.points=moved;
 if(row.status==='da_xac_nhan')row.status='chua_kiem_tra';
 recordTakeoffRevision(row,'move_markup',{points:previous,status:previousStatus},{points:moved,status:row.status});
 render();toast('Đã di chuyển dấu đo, giữ nguyên khối lượng; cần duyệt lại');
}

function finishPolygon(){if(!['area','perimeter'].includes(tool))return;if(draft.length<3){toast('Cần ít nhất 3 điểm',true);return}if(!polygonIsSimple(draft)){toast('Vùng bị tự cắt, cạnh trùng hoặc suy biến',true);return}commitMeasure(draft.splice(0))}function cancelCurrent(){draft=[];calDraft=[];render()}function commitMeasure(points){try{const [qty,unit]=valueFor(tool,points,calibration?.metresPerPixel);if(!finite(qty)||qty<0)throw Error('Kết quả không hợp lệ');if((tool==='area'||tool==='perimeter')&&!polygonIsSimple(points))throw Error('Vùng đo không hợp lệ');if(measurements.length>=MAX_ROWS)throw Error('Đã đạt giới hạn 20.000 phép đo');snapshot();measurements.push({id:'P-'+String(Date.now())+'-'+measurements.length,source:'PDF/Image',sourceFile:pdfName,page:pageIndex+1,classification:el('group').value.trim()||'CHUA_PHAN_LOAI',description:el('description').value.trim(),kind:tool,points,scale:calibration.metresPerPixel,quantity:qty,unit,status:'chua_kiem_tra'});selected=measurements[measurements.length-1].id;render();toast('Đã thêm '+qty.toFixed(3)+' '+unit)}catch(e){toast(e.message,true)}}
function drawLine(pts,closed,color,width=2){if(!pts.length)return;ctx.beginPath();ctx.moveTo(...pts[0]);pts.slice(1).forEach(p=>ctx.lineTo(...p));if(closed)ctx.closePath();ctx.strokeStyle=color;ctx.lineWidth=width;ctx.stroke();pts.forEach(p=>{ctx.beginPath();ctx.arc(p[0],p[1],3.5,0,Math.PI*2);ctx.fillStyle=color;ctx.fill()})}
let scanCandidates=[],scanPosition=0,scanSource='';
let scanPageStore={};
let auditTrail=[];
let comparisonBaseline=null;
function restoreScanPage(){
 const record=scanPageStore[String(pageIndex+1)];
 scanCandidates=record?.lines||[];
 scanPosition=0;
 scanSource=record?.source||'';
}

function scanProgress(){el('scanProgress').textContent=scanCandidates.length?`Ứng viên ${scanPosition+1}/${scanCandidates.length} · trang ${pageIndex+1} · chỉ đo sau khi hiệu chỉnh tỷ lệ`:'Chưa có nét ứng viên trên trang này.'}
function scanPrevious(){if(scanCandidates.length)scanPosition=(scanPosition+scanCandidates.length-1)%scanCandidates.length;scanProgress();render()}
function scanNext(){if(scanCandidates.length)scanPosition=(scanPosition+1)%scanCandidates.length;scanProgress();render()}
function clearScanCandidates(){delete scanPageStore[String(pageIndex+1)];restoreScanPage();scanProgress();render()}
function acceptScanLine(){
 if(!scanCandidates.length){toast('Chưa chọn nét scan',true);return}
 if(!calibration?.metresPerPixel||!finite(calibration.metresPerPixel)||calibration.metresPerPixel<=0){toast('Cần hiệu chỉnh tỷ lệ trang trước khi xác nhận',true);return}
 const pts=scanCandidates[scanPosition];const q=dist(pts[0],pts[1])*calibration.metresPerPixel;
 if(!finite(q)||q<=0){toast('Nét không hợp lệ',true);return}
 const id='SCAN-'+scanSource+'-'+(pageIndex+1)+'-'+scanPosition;
 if(measurements.some(x=>x.id===id)){toast('Nét đã được xác nhận, không đo trùng',true);return}
 snapshot();measurements.push({id,source:'PDF_SCAN_REVIEWED',sourceFile:scanSource,page:pageIndex+1,classification:el('scanClass').value.trim()||'Nét đã xác nhận',description:'Nét scan do người dùng xác nhận',kind:'length',quantity:q,unit:'m',scale:calibration.metresPerPixel,status:'chua_kiem_tra',points:pts});
 render();toast('Đã ghi một nét; vẫn cần kiểm tra ý nghĩa cấu kiện')
}
el('scanCandidatesFile').onchange=async e=>{try{
 const f=e.target.files[0];if(!f)return;const obj=JSON.parse(await f.text());
 if(obj.status!=='CANDIDATE_ONLY_NOT_VERIFIED'||!Array.isArray(obj.pages)||obj.pages.length>100)throw Error('Dữ liệu scan không đúng schema');
 const page=obj.pages.find(p=>p.page===pageIndex+1);if(!page)throw Error('Không tìm thấy ứng viên cho trang hiện tại');
 const width=Number(page.width_px),height=Number(page.height_px);
 if(!finite(width)||!finite(height)||width<=0||height<=0||!Array.isArray(page.candidate_lines_px)||page.candidate_lines_px.length>800)throw Error('Số nét hoặc kích thước không hợp lệ');
 if(!image.naturalWidth||!image.naturalHeight)throw Error('Hãy mở ảnh trang trước khi nhập scan');
 const sx=image.naturalWidth/width,sy=image.naturalHeight/height;
 const lines=page.candidate_lines_px.map(a=>{if(!Array.isArray(a)||a.length!==4||a.some(v=>!finite(Number(v))))throw Error('Tọa độ nét không hợp lệ');return [[Number(a[0])*sx,Number(a[1])*sy],[Number(a[2])*sx,Number(a[3])*sy]]});
 scanPageStore[String(pageIndex+1)]={lines,source:obj.source||f.name};restoreScanPage();scanProgress();render();toast('Đã nạp '+lines.length+' nét ứng viên cho trang '+(pageIndex+1)+', chưa tính khối lượng');
 }catch(err){toast(err.message,true)}};
function render(){if(image.complete&&image.naturalWidth){if(canvas.width!==image.naturalWidth)canvas.width=image.naturalWidth;if(canvas.height!==image.naturalHeight)canvas.height=image.naturalHeight;ctx.clearRect(0,0,canvas.width,canvas.height);ctx.drawImage(image,0,0)}else{canvas.width=1000;canvas.height=700;ctx.fillStyle='#fff';ctx.fillRect(0,0,1000,700);ctx.fillStyle='#566';ctx.fillText('Chọn ảnh bản vẽ',30,30)}for(const m of measurements.filter(x=>(x.page||1)===pageIndex+1)){const color=m.id===selected?'#0784fa':m.status==='da_xac_nhan'?'#15945a':'#de3144';drawLine(m.points,m.kind==='area'||m.kind==='perimeter',color,2.5);const p=m.points[0];ctx.fillStyle=color;ctx.font='bold 14px sans-serif';ctx.fillText(m.quantity.toFixed(2)+' '+m.unit,p[0]+8,p[1]-8)}drawLine(draft,tool==='area'||tool==='perimeter','#ff9b00',2);if(calibration){drawLine(calibration.points,false,'#18875d',2);el('calStatus').textContent='Đã hiệu chỉnh: '+calibration.realMetres+' m / '+dist(...calibration.points).toFixed(1)+' px'}else el('calStatus').textContent='Chưa hiệu chỉnh — cần hai điểm có chiều dài thực';drawLine(calDraft,false,'#2294bb');renderRows()}
const _renderBase=render;render=function(){_renderBase();if(mode==='pdf'&&image.complete&&image.naturalWidth&&scanCandidates.length){const pts=scanCandidates[scanPosition];drawLine(pts,false,'#ed6f00',5)}scanProgress()};
function renderRows(){const query=(el('qtoFilter')?.value||'').trim().toLocaleLowerCase('vi');const statusFilter=el('qtoStatusFilter')?.value||'';const rows=(mode==='model'?modelRows:measurements).filter(x=>(!statusFilter||x.status===statusFilter)&&(!query||[x.id,x.classification,x.description,x.sourceFile].join(' ').toLocaleLowerCase('vi').includes(query)));el('rows').textContent='';for(const r of rows.slice(0,2500)){const tr=document.createElement('tr');if(r.id===selected)tr.className='selected';for(const v of [r.classification+' / '+(r.description||r.id),Number(r.quantity).toLocaleString('vi-VN',{maximumFractionDigits:3}),r.unit,r.status]){const td=document.createElement('td');td.textContent=String(v);tr.append(td)}tr.onclick=()=>{selected=r.id;el('reviewStatus').value=r.status;el('editGroup').value=r.classification||'';el('editDescription').value=r.description||'';el('editNote').value=r.reviewNote||'';el('canonicalLink').value=canonicalKey(r);if(mode==='pdf'&&r.page&&r.page!==pageIndex+1&&r.page<=pages.length){pages[pageIndex].calibration=calibration;pageIndex=r.page-1;restoreScanPage();loadPage(true);selected=r.id;}render()};el('rows').append(tr)}el('itemCount').textContent=(measurements.length+modelRows.length).toLocaleString('vi-VN');el('reviewCount').textContent=[...measurements,...modelRows].filter(r=>r.status==='chua_kiem_tra').length;el('viewCaption').textContent=mode==='pdf'?'Nhấp trên bản vẽ để đo. Mỗi phép đo có mã ID, nguồn và trạng thái.':'Số liệu 3D là thể tích tham chiếu từ bảng Solid, chưa phải hình học IFC native.'}
function recordTakeoffRevision(record,action,before,after){
 auditTrail.push({at:new Date().toISOString(),entityId:record.id,source:record.source,sourceFile:record.sourceFile||'',page:record.page||'',action,before,after});
 if(auditTrail.length>20000)auditTrail.shift();
}
function updateSelectedTakeoff(){
 const r=[...measurements,...modelRows].find(x=>x.id===selected);
 if(!r){toast('Hãy chọn dòng QTO cần chỉnh',true);return}
 const group=el('editGroup').value.trim(),description=el('editDescription').value.trim(),note=el('editNote').value.trim();
 if(!group||group.length>120||description.length>400||note.length>1000){toast('Mã công tác bắt buộc; diễn giải hoặc nhận xét quá dài',true);return}
 const before={classification:r.classification,description:r.description,reviewNote:r.reviewNote||''};
 const after={classification:group,description,reviewNote:note};
 snapshot();r.classification=group;r.description=description;r.reviewNote=note;
 recordTakeoffRevision(r,'update_metadata',before,after);render();toast('Đã cập nhật QTO và ghi nhật ký');
}
function exportAuditTrail(){
 download('TBT360_QTO_NHAT_KY.csv',csv([['Thời gian','ID','Nguồn','Tệp','Trang','Thao tác','Trước','Sau'],...auditTrail.map(x=>[x.at,x.entityId,x.source,x.sourceFile,x.page,x.action,JSON.stringify(x.before),JSON.stringify(x.after)])]),'text/csv;charset=utf-8');
}
function applyReview(){const all=[...measurements,...modelRows],r=all.find(x=>x.id===selected);if(!r){toast('Hãy chọn một dòng',true);return}const next=el('reviewStatus').value;if(next==='da_xac_nhan'&&(!r.classification||!r.unit||!Number.isFinite(Number(r.quantity))||Number(r.quantity)<0)){toast('Không thể duyệt dòng thiếu phân loại, đơn vị hoặc có số liệu lỗi',true);return}snapshot();const before={status:r.status};r.status=next;recordTakeoffRevision(r,'review',before,{status:next});render();toast('Đã cập nhật trạng thái và ghi nhật ký')}
function removeSelected(){const i=measurements.findIndex(x=>x.id===selected);if(i<0){toast('Chỉ xóa được phép đo được tạo từ bản vẽ',true);return}const row=measurements[i];snapshot();recordTakeoffRevision(row,'delete_measurement',{quantity:row.quantity,unit:row.unit,points:row.points,status:row.status},null);measurements.splice(i,1);selected=null;render();toast('Đã xóa dấu đo; có thể Undo và kiểm tra nhật ký')}
function fitCanvas(){zoom=Math.min((viewport.clientWidth-40)/canvas.width,(viewport.clientHeight-35)/canvas.height,1.5);setCanvasZoom()}function setZoom(f){zoom=Math.min(5,Math.max(.2,zoom*f));setCanvasZoom()}function setCanvasZoom(){canvas.style.width=Math.round(canvas.width*zoom)+'px';canvas.style.height=Math.round(canvas.height*zoom)+'px';el('zoomLabel').textContent=Math.round(zoom*100)+'%'}
function download(name,content,type='text/plain'){const url=URL.createObjectURL(new Blob([content],{type}));const a=document.createElement('a');a.href=url;a.download=name;document.body.appendChild(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(url),1000)}function exportMarkedImage(){canvas.toBlob(blob=>{const url=URL.createObjectURL(blob);const a=document.createElement('a');a.href=url;a.download='TBT360_DANH_DAU.png';a.click();setTimeout(()=>URL.revokeObjectURL(url),1000)},'image/png')}
function csvVal(x){const raw=String(x??'');const s=/^[\t\r\n ]*[=+@-]/.test(raw)?'\''+raw:raw;return '"'+s.replace(/"/g,'""')+'"'}function csv(rows){return '\ufeff'+rows.map(row=>row.map(csvVal).join(',')).join('\r\n')};function exportCSV(){download('TBT360_QTO_CHI_TIET.csv',csv([['ID','Nguồn','Tệp','Trang','Nhóm','Diễn giải','Loại đo','Giá trị','Đơn vị','Trạng thái','Tỷ lệ m/px'],...[...measurements,...modelRows].map(r=>[r.id,r.source,r.sourceFile,r.page||'',r.classification,r.description,r.kind,r.quantity,r.unit,r.status,r.scale])]),'text/csv;charset=utf-8')}
function exportSummary(){const map=new Map();for(const r of [...measurements,...modelRows]){if(r.status==='bo_qua')continue;const key=JSON.stringify([r.source,r.classification,r.unit,r.kind]);map.set(key,(map.get(key)||0)+Number(r.quantity))}download('TBT360_QTO_TONG_HOP.csv',csv([['Nguồn','Nhóm','Đơn vị','Loại đo','Khối lượng','Trạng thái'],...[...map.entries()].map(([k,v])=>[...JSON.parse(k),v,'CHUA_NGHIEM_THU'])]),'text/csv;charset=utf-8')}
// Stable inventory key: sources and units are always isolated, never blindly added.
function approvedInventory(){
 const aggregate=new Map(),seen=new Set(),duplicates=[];
 for(const r of [...measurements,...modelRows]){
  if(r.status!=='da_xac_nhan')continue;
  if(!Number.isFinite(Number(r.quantity))||Number(r.quantity)<0||!r.unit)continue;
  const identity=[r.source,r.sourceFile,r.page||'',r.id].join('|');
  if(seen.has(identity)){duplicates.push(identity);continue}seen.add(identity);
  const key=JSON.stringify([r.source,r.classification,r.kind,r.unit]);
  const item=aggregate.get(key)||{source:r.source,group:r.classification,kind:r.kind,unit:r.unit,quantity:0,count:0,ids:[]};
  item.quantity+=Number(r.quantity);item.count++;item.ids.push(r.id);aggregate.set(key,item);
 }
 return {rows:[...aggregate.values()],duplicates};
}
function showInventory(){
 const result=approvedInventory();const count=result.rows.reduce((n,r)=>n+r.count,0);
 el('inventorySummary').textContent='Đã duyệt: '+count+' phép đo / '+result.rows.length+' nhóm theo nguồn–hạng mục–loại–đơn vị; '+result.duplicates.length+' ID trùng bị loại. Không cộng chéo nguồn.';
 toast('Bảng tổng hợp: '+result.rows.length+' nhóm, '+count+' phép đo được duyệt.',!!result.duplicates.length);
 return result;
}
function exportApproved(){
 const result=approvedInventory();
 if(!result.rows.length){toast('Chưa có phép đo được duyệt để xuất',true);return}
 download('TBT360_QTO_DA_DUYET.csv',csv([['Nguồn','Mã công tác','Loại đo','Đơn vị','Khối lượng','Số phép đo','IDs'],...result.rows.map(r=>[r.source,r.group,r.kind,r.unit,r.quantity,r.count,r.ids.join(';')])]),'text/csv;charset=utf-8');
}
// R128: Reconciliation is advisory. Never silently merge 2D/PDF and 3D.
function canonicalKey(row){
 const raw=String(row.canonicalId||row.canonical_id||'').trim();
 return /^[A-Za-z0-9_.:-]{3,128}$/.test(raw)?raw:'';
}
function validateBaseline(candidate){
 if(candidate==null)return null;
 if(!Array.isArray(candidate)||candidate.length>MAX_ROWS)throw Error('Bản đối chiếu không hợp lệ');
 return candidate.map(r=>{
  if(!r||typeof r.id!=='string'||r.id.length>256||!finite(Number(r.quantity))||Number(r.quantity)<0||typeof r.unit!=='string'||r.unit.length>40)throw Error('Dòng baseline sai dữ liệu');
  return {id:r.id,canonicalId:canonicalKey(r),source:String(r.source||'').slice(0,128),classification:String(r.classification||'').slice(0,128),kind:String(r.kind||''),unit:r.unit,quantity:Number(r.quantity),status:String(r.status||''),sourceFile:String(r.sourceFile||'').slice(0,256),page:r.page||''};
 });
}
function comparisonRows(rows,baseline){
 const id=r=>[r.source,r.sourceFile,r.page||'',r.id].join('|');
 const current=new Map(rows.map(r=>[id(r),r]));
 const old=new Map((baseline||[]).map(r=>[id(r),r]));
 const differences=[];
 for(const key of new Set([...current.keys(),...old.keys()])){
  const a=old.get(key),b=current.get(key);
  const change=!a?'ADDED':!b?'REMOVED':a.unit!==b.unit||a.kind!==b.kind?'UNIT_OR_TYPE_CHANGED':Number(a.quantity)!==Number(b.quantity)||a.status!==b.status||a.classification!==b.classification?'CHANGED':'UNCHANGED';
  if(change!=='UNCHANGED')differences.push({key,change,before:a,after:b,delta:a&&b&&a.unit===b.unit&&a.kind===b.kind?Number(b.quantity)-Number(a.quantity):null});
 }
 return differences;
}
function reconcileSources(rows){
 const groups=new Map(),warnings=[];
 for(const r of rows){
  if(r.status!=='da_xac_nhan')continue;
  const key=canonicalKey(r);if(!key)continue;
  const group=groups.get(key)||[];group.push(r);groups.set(key,group);
 }
 for(const [key,group] of groups){
  const sources=new Set(group.map(r=>r.source));
  if(sources.size>1)warnings.push({canonicalId:key,rows:group,reason:'SAME_CANONICAL_ID_MULTIPLE_SOURCES'});
  else if(group.length>1)warnings.push({canonicalId:key,rows:group,reason:'SAME_CANONICAL_ID_REPEATED'});
 }
 return warnings;
}
function captureBaseline(){
 snapshot();comparisonBaseline=validateBaseline([...measurements,...modelRows]);
 el('revisionStatus').textContent='Đã chụp mốc '+comparisonBaseline.length+' dòng QTO';
 toast('Đã chụp mốc đối chiếu trong dự án; lưu JSON để bảo toàn');
}
function showRevision(){
 if(!comparisonBaseline){toast('Hãy chụp mốc hoặc nhập dự án có mốc so sánh',true);return}
 const rows=[...measurements,...modelRows],changes=comparisonRows(rows,comparisonBaseline),warnings=reconcileSources(rows);
 el('revisionStatus').textContent=changes.length+' thay đổi; '+warnings.length+' mã cấu kiện có nguy cơ trùng nhiều nguồn. Không tự cộng.';
 download('TBT360_QTO_SO_SANH_PHIEN_BAN.csv',csv([['Khóa dòng','Thay đổi','Giá trị cũ','Giá trị mới','Đơn vị cũ','Đơn vị mới','Chênh lệch cùng đơn vị'],...changes.map(c=>[c.key,c.change,c.before?.quantity??'',c.after?.quantity??'',c.before?.unit??'',c.after?.unit??'',c.delta??''])]),'text/csv;charset=utf-8');
 download('TBT360_QTO_CANH_BAO_TRUNG.csv',csv([['Canonical ID','Lý do','Nguồn','IDs'],...warnings.map(w=>[w.canonicalId,w.reason,w.rows.map(r=>r.source).join(';'),w.rows.map(r=>r.id).join(';')])]),'text/csv;charset=utf-8');
}
function updateCanonicalLink(){
 const r=[...measurements,...modelRows].find(x=>x.id===selected);if(!r){toast('Chọn một dòng trước',true);return}
 const raw=el('canonicalLink').value.trim();
 if(raw&&!/^[A-Za-z0-9_.:-]{3,128}$/.test(raw)){toast('Mã canonical không hợp lệ',true);return}
 snapshot();const before={canonicalId:canonicalKey(r)};r.canonicalId=raw;
 recordTakeoffRevision(r,'link_canonical',before,{canonicalId:raw});render();toast('Đã lưu liên kết. Cần xác minh ID cùng cấu kiện trước khi duyệt QTO');
}
function saveProject(){download('TBT360_QTO_PROJECT.json',JSON.stringify({schema:'tbt360-qto-v1',savedAt:new Date().toISOString(),pdf_render_dpi:Number(el('pdfDpi').value),pdfName,imageData:imgData,pages,pageIndex,measurements,modelRows,calibration,scanPageStore,auditTrail,comparisonBaseline},null,2),'application/json')}
el('projectFile').onchange=async e=>{try{const o=JSON.parse(await e.target.files[0].text());if(o.schema!=='tbt360-qto-v1'||!Array.isArray(o.measurements)||o.measurements.length>MAX_ROWS||!Array.isArray(o.modelRows))throw Error('Dự án sai phiên bản hoặc quá lớn');for(const r of o.measurements){if(!['length','area','perimeter','count'].includes(r.kind)||!Array.isArray(r.points)||r.points.length>20000||r.points.some(p=>!Array.isArray(p)||p.length!==2||p.some(x=>!finite(x))))throw Error('Dữ liệu phép đo không hợp lệ')}snapshot();measurements=o.measurements;modelRows=o.modelRows;pages=Array.isArray(o.pages)&&o.pages.length>0&&o.pages.length<=100&&o.pages.every(p=>typeof p.name==='string'&&typeof p.data==='string'&&(p.data.startsWith('data:image/')||p.data.startsWith('assets/')))?o.pages:[{name:o.pdfName||'Trang 1',data:o.imageData||'assets/ban_ve_mau.png',calibration:o.calibration}];pageIndex=Math.min(Math.max(0,Number(o.pageIndex)||0),pages.length-1);calibration=pages[pageIndex].calibration||null;pdfName=pages[pageIndex].name;if(o.pdf_render_dpi)el('pdfDpi').value=o.pdf_render_dpi;auditTrail=Array.isArray(o.auditTrail)&&o.auditTrail.length<=20000?o.auditTrail.filter(x=>x&&typeof x.entityId==='string'&&typeof x.action==='string'):[];comparisonBaseline=validateBaseline(o.comparisonBaseline);scanPageStore=o.scanPageStore&&typeof o.scanPageStore==='object'?o.scanPageStore:{};
 for(const [page,record] of Object.entries(scanPageStore)){
  if(!/^[1-9][0-9]*$/.test(page)||Number(page)>pages.length||!record||typeof record.source!=='string'||record.source.length>512||!Array.isArray(record.lines)||record.lines.length>800||record.lines.some(line=>!Array.isArray(line)||line.length!==2||line.some(point=>!Array.isArray(point)||point.length!==2||point.some(value=>!finite(value)))))throw Error('Dữ liệu nét scan lưu trữ không hợp lệ');
 }
 restoreScanPage();selected=null;loadPage(true);toast('Đã mở dự án '+measurements.length+' phép đo')}catch(err){toast(err.message,true)}};
el('fileImage').onchange=async e=>{try{const files=[...e.target.files].sort((a,b)=>a.name.localeCompare(b.name,undefined,{numeric:true}));if(!files.length)return;if(files.length>100||files.some(f=>!f.type.startsWith('image/')||f.size>15*1024*1024))throw Error('Tối đa 100 trang, 15 MB mỗi ảnh');const read=f=>new Promise((resolve,reject)=>{const r=new FileReader();r.onload=()=>resolve({name:f.name,data:r.result,calibration:null});r.onerror=reject;r.readAsDataURL(f)});const incoming=await Promise.all(files.map(read));snapshot();pages=incoming;pageIndex=0;measurements=[];calibration=null;selected=null;loadPage(true);toast('Đã nhập '+pages.length+' trang, cần hiệu chỉnh riêng từng trang')}catch(err){toast(err.message,true)}};
function parseCSV(s){const out=[];let field='',row=[],quote=false;for(let i=0;i<s.length;i++){const c=s[i];if(quote){if(c==='"'&&s[i+1]==='"'){field+='"';i++}else if(c==='"')quote=false;else field+=c}else if(c==='"')quote=true;else if(c===','){row.push(field);field=''}else if(c==='\n'){row.push(field.replace(/\r$/,''));out.push(row);row=[];field=''}else field+=c}if(field||row.length){row.push(field);out.push(row)}return out}
el('volumeFile').onchange=async e=>{try{const f=e.target.files[0],raw=await f.text(),rows=parseCSV(raw),keys=rows.shift().map(x=>x.replace(/^\ufeff/,''));const get=(r,k)=>r[keys.indexOf(k)];if(!keys.includes('canonical_id')||!keys.includes('volume_m3'))throw Error('CSV cần cột canonical_id và volume_m3');snapshot();modelRows=rows.filter(r=>r.length>=keys.length).slice(0,100000).map((r,i)=>{let vol=Number(get(r,'volume_m3'));if(!finite(vol)||vol<0)throw Error('Thể tích sai ở dòng '+(i+2));return{id:'S-'+get(r,'canonical_id'),source:'STEP_VOLUME_MAP',sourceFile:f.name,classification:get(r,'ifc_candidate_class')||'SOLID',description:get(r,'canonical_id'),kind:'volume',quantity:vol,unit:'m³',status:'chua_kiem_tra'}});switchMode('model');toast('Đã nhập '+modelRows.length+' thể tích. Cần đối chiếu nguồn và trùng ID')}catch(err){toast(err.message,true)}};
function showModelRows(){switchMode('model')}

function showAudit(){
 const all=[...measurements,...modelRows], seen=new Set(), duplicates=[], invalid=[];
 for(const r of all){
  const key=[r.source,r.sourceFile,r.id].join('|');
  if(seen.has(key))duplicates.push(r.id);seen.add(key);
  if(!Number.isFinite(Number(r.quantity))||Number(r.quantity)<0||!r.unit)invalid.push(r.id);
 }
 const groups=new Map();for(const r of all){if(r.status==='bo_qua')continue;
  const key=[r.source,r.classification,r.kind,r.unit].join('|');
  groups.set(key,(groups.get(key)||0)+Number(r.quantity));
 }
 toast('Kiểm tra: '+all.length+' dòng; '+duplicates.length+' ID trùng trong nguồn; '+invalid.length+' số liệu lỗi; '+groups.size+' nhóm. Không cộng chéo nguồn/đơn vị.',!!(duplicates.length||invalid.length));
 return {total:all.length,duplicates,invalid,groups:groups.size};
}
el('ifcFile').onchange=async e=>{try{
 const f=e.target.files[0];if(!f)return;
 const rows=parseCSV(await f.text()), keys=rows.shift().map(x=>x.replace(/^\ufeff/,''));
 for(const k of ['global_id','entity_type','quantity','unit'])if(!keys.includes(k))throw Error('Thiếu cột '+k);
 const get=(r,k)=>r[keys.indexOf(k)];const imported=[];const seen=new Set();
 for(const r of rows){if(r.length<keys.length)continue;
  const id=get(r,'global_id'),typ=get(r,'entity_type'),qty=Number(get(r,'quantity')),unit=get(r,'unit');
  if(!id||seen.has(id+'|'+unit)||!Number.isFinite(qty)||qty<0||!['m','m²','m³','cái'].includes(unit))throw Error('IFC QTO sai ID/đơn vị/giá trị hoặc trùng: '+id);
  seen.add(id+'|'+unit);imported.push({id:'IFC-'+id+'-'+unit,source:'IFC_VALIDATED',sourceFile:f.name,classification:typ,description:id,kind:'ifc_quantity',quantity:qty,unit,status:'chua_kiem_tra'});
 }
 if(imported.length>20000)throw Error('Quá giới hạn 20.000 dòng');
 snapshot();modelRows=modelRows.filter(x=>x.source!=='IFC_VALIDATED').concat(imported);switchMode('model');toast('Đã nhập '+imported.length+' dòng IFC QTO; cần duyệt trước khi dùng');
 }catch(err){toast(err.message,true)}};

loadPage(true);


// R129: Export a verifiable, per-page takeoff evidence manifest alongside the markup PNG.
function exportMeasurementEvidence(){
  const pageRows=measurements.filter(row=>(row.page||1)===pageIndex+1);
  if(!pageRows.length){toast('Trang này chưa có phép đo để xuất bằng chứng',true);return;}
  const page=pages[pageIndex];
  const evidence={schema:'tbt360-takeoff-evidence-v1',page:pageIndex+1,
    source:page.name,calibration:page.calibration||calibration,
    measurements:pageRows.map(row=>({id:row.id,kind:row.kind,points:row.points,
      quantity:row.quantity,unit:row.unit,status:row.status,
      classification:row.classification,canonicalId:row.canonicalId||null})),
    exportTime:new Date().toISOString(),
    note:'Screenshot is from current canvas state. Not native TBT360 EXE evidence.'};
  download('TBT360_BANG_CHUNG_TRANG_'+(pageIndex+1)+'.json',
    JSON.stringify(evidence,null,2),'application/json');
  exportMarkedImage();
  toast('Đã xuất PNG dấu đo + JSON có ID, tỷ lệ, nguồn trang');
}
