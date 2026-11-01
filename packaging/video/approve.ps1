# Plays the admin: approves the pending demo payment in the license mock (1 year).
$repo = "C:\Users\user\HTL-Content-Guard\Blocker-app-for-pc-ccr-00759c63-vxxesd"
Get-CimInstance Win32_Process -Filter "Name='node.exe'" | Where-Object { $_.CommandLine -like "*src/server.mjs*" } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
Start-Sleep -Milliseconds 600
$f = "$repo\.dev\demo\backend\db.json"
$py = @'
import json,sys,datetime
f=sys.argv[1];d=json.load(open(f,encoding="utf-8"))
until=(datetime.datetime.utcnow()+datetime.timedelta(days=365)).isoformat()+"Z"
for a in d["accounts"].values():
    a["plan"]="premium"; a["premium_until"]=until
for r in d.get("purchase_requests",[]): r["status"]="approved"
json.dump(d,open(f,"w",encoding="utf-8"),ensure_ascii=False)
print("approved until",until)
'@
python -I -c $py $f
$env:PORT = "8788"; $env:DATA_DIR = "..\.dev\demo\backend"
Start-Process node -ArgumentList "src/server.mjs" -WorkingDirectory "$repo\backend" -WindowStyle Hidden -RedirectStandardOutput "C:\Users\user\AppData\Local\Temp\cgvideo\mock2.log"
