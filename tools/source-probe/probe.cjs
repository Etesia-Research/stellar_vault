// Unsigned read-only testnet probe, plus the one mainnet read (`activation`). No signing or submission code.
// Run from etesia-vault/: node tools/source-probe/probe.cjs <reads|tx|activation> <input.json> <evidence.json> [args]
//   reads                 R1-R7 and the v3 copy's config(), once, after the v3 deploy
//   tx <hash>...          verbatim getTransaction (xdrFormat json) and the JS decode of the archived meta
//   activation [rpc-url]  first mainnet ledger whose header reports protocol 30
const fs = require("node:fs");
const crypto = require("node:crypto");
const {execFileSync} = require("node:child_process");
const S = require("@stellar/stellar-sdk");
const [mode, inputPath, evidencePath, ...rest] = process.argv.slice(2);
if (!["reads", "tx", "activation"].includes(mode) || !inputPath || !evidencePath) throw new Error("usage: probe.cjs <reads|tx|activation> <input.json> <evidence.json> [args]");
const input = JSON.parse(fs.readFileSync(inputPath));
const LABEL = "testnet rehearsal evidence; Etesia-deployed copy of Soroswap's mainnet v3 WASM; no mainnet claim";
const EPOCH = "pre-2026-12-16-reset";
// Indented, but arrays of plain values (the R4 rows, args, reserves) stay on one line.
const json = value => JSON.stringify(value, function(k,v){const raw=this[k]; return typeof v === "bigint" ? v.toString() : raw instanceof Uint8Array ? Buffer.from(raw).toString("hex") : v;}, 2)
 .replace(/\[\n\s+([^\[\]{}]*?)\n\s+\]/g, (_,body) => "["+body.split(/,\n\s+/).join(", ")+"]");
const net = mode === "activation" ? {rpc: rest[0] || input.mainnet.rpc, passphrase: input.mainnet.passphrase} : {rpc: input.rpc, passphrase: input.passphrase};
const endpoint = new URL(net.rpc).origin; // never the path: keyed provider URLs carry the key there
let account;

async function rpc(method, params={}) {
 const response=await fetch(net.rpc,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({jsonrpc:"2.0",id:1,method,params})});
 if(!response.ok)throw new Error(method+" RPC HTTP "+response.status);
 const payload=await response.json(); if(payload.error)throw new Error(method+" "+JSON.stringify(payload.error));
 return payload.result;
}
const closes = new Map();
async function close(seq) { // ledger close time, seconds
 if(!closes.has(seq)){const l=(await rpc("getLedgers",{startLedger:seq,pagination:{limit:1}})).ledgers[0];
  if(l.sequence!==seq)throw new Error("getLedgers returned "+l.sequence+" for "+seq); closes.set(seq,Number(l.ledgerCloseTime));}
 return closes.get(seq);
}
const utc = s => new Date(s*1000).toISOString();
const reads = [];
async function call(id, method, args=[]) {
 const tx=new S.TransactionBuilder(account,{fee:"100",networkPassphrase:net.passphrase}).addOperation(new S.Contract(id).call(method,...args)).setTimeout(60).build();
 const result=await rpc("simulateTransaction",{transaction:tx.toXDR(),authMode:"record"});
 const row={captured_at:new Date().toISOString(),id,method,args:args.map(a=>a.toXDR("base64")),ledger:result.latestLedger,
  endpoint,ledger_close:utc(await close(result.latestLedger)),auth_mode:"record"};
 if(result.error){row.error=result.error;reads.push(row);throw new Error(result.error);}
 row.value=S.scValToNative(S.xdr.ScVal.fromXDR(result.results[0].xdr,"base64"));
 row.auth=result.results[0].auth||[];
 const res=S.xdr.SorobanTransactionData.fromXDR(result.transactionData,"base64").resources; // RPC 30 returns no `cost`
 row.resources={min_resource_fee:result.minResourceFee,instructions:res.instructions,disk_read_bytes:res.diskReadBytes,write_bytes:res.writeBytes};
 row.restore_required=Boolean(result.restorePreamble);
 reads.push(row); return row;
}
async function identity(id){
 const r=await rpc("getLedgerEntries",{keys:[new S.Contract(id).getFootprint().toXDR("base64")]});
 const raw=r.entries[0], entry=S.xdr.LedgerEntryData.fromXDR(raw.xdr,"base64");
 const exe=entry.contractData.val.instance.executable;
 const out={id,ledger:r.latestLedger,endpoint,instance_last_modified:raw.lastModifiedLedgerSeq,instance_live_until:raw.liveUntilLedgerSeq};
 if(exe.type!=="contractExecutableWasm")return {...out,executable:"stellar-asset"};
 const hash=exe.wasmHash;
 const c=await rpc("getLedgerEntries",{keys:[S.xdr.LedgerKey.contractCode(new S.xdr.LedgerKeyContractCode({hash})).toXDR("base64")]});
 const wasm=Buffer.from(S.xdr.LedgerEntryData.fromXDR(c.entries[0].xdr,"base64").contractCode.code);
 fs.mkdirSync("artifacts/local/upstream",{recursive:true});
 fs.writeFileSync("artifacts/local/upstream/"+hash.toString()+".wasm",wasm);
 const sha256=crypto.createHash("sha256").update(wasm).digest("hex");
 if(sha256!==hash.toString())throw new Error("code entry hash mismatch for "+id);
 return {...out,executable:"wasm",wasm_hash:sha256,bytes:wasm.length,code_last_modified:c.entries[0].lastModifiedLedgerSeq,code_live_until:c.entries[0].liveUntilLedgerSeq};
}
const feedKey=(kind,id)=>S.xdr.ScVal.scvVec([S.nativeToScVal(kind,{type:"symbol"}),kind==="Stellar"?new S.Address(id).toScVal():S.nativeToScVal(id,{type:"symbol"})]);
const address = id => new S.Address(id).toScVal();
const u32 = n => S.nativeToScVal(n,{type:"u32"});

function sh(file, ...args) { return execFileSync(file, args, {encoding: "utf8"}).trim(); }
function meta(captured_at, seqs) { // seqs: the ledgers this key's content comes from
 const ledgers={first:Math.min(...seqs),last:Math.max(...seqs)};
 const git=dir=>({commit:sh("git","-C",dir,"rev-parse","HEAD"),dirty:sh("git","-C",dir,"status","--porcelain")!==""});
 const py=require("node:path").resolve("../etesia-quant/.venv/bin/python");
 return {captured_at,endpoint,ledgers,
  revisions:{vault:git("."),quant:git("../etesia-quant"),script_sha256:crypto.createHash("sha256").update(fs.readFileSync(__filename)).digest("hex")},
  tools:{stellar_cli:sh(".tooling/bin/stellar","--version").split("\n")[0],node:process.version,
   stellar_sdk_js:JSON.parse(fs.readFileSync(require.resolve("@stellar/stellar-sdk").replace(/lib\/.*$/,"package.json"))).version,
   python:sh(py,"--version"),stellar_sdk_py:sh(py,"-c","import stellar_sdk;print(stellar_sdk.__version__)")}};
}
function load() { return fs.existsSync(evidencePath) ? JSON.parse(fs.readFileSync(evidencePath)) : {label: LABEL, network_epoch: EPOCH}; }
function absent(keys) { const ev=load(); for(const k of keys)if(k in ev)throw new Error(k+" is already recorded in "+evidencePath+"; each key is written once"); }
function write(values) { absent(Object.keys(values)); fs.writeFileSync(evidencePath,json({...load(),...values})+"\n"); }

async function reflectorScan(id) { // R4: every update event the RPC still holds
 const health=await rpc("getHealth"), target=health.latestLedger, start=health.oldestLedger+20;
 const topic=s=>S.nativeToScVal(s,{type:"symbol"}).toXDR("base64");
 const filters=[{type:"contract",contractIds:[id],topics:[[topic("REFLECTOR"),topic("update"),"*"]]}];
 const events=[]; let params={startLedger:start,filters,pagination:{limit:1000}}, pages=0, last=-1;
 for(;;){
  const r=await rpc("getEvents",params); pages++; events.push(...r.events);
  const at=Number(BigInt(r.cursor.split("-")[0])>>32n); // never stop on a short page; stop when the cursor reaches the target
  if(at>=target)break; if(at<=last)throw new Error("getEvents cursor did not advance at ledger "+at);
  last=at; params={filters,pagination:{cursor:r.cursor,limit:1000}};
 }
 const byRound=new Map(); let duplicates=0;
 for(const e of events){
  const round=Number(S.scValToNative(S.xdr.ScVal.fromXDR(e.topic[2],"base64")))/1000;
  if(byRound.has(round)){duplicates++;continue;}
  const data=S.scValToNative(S.xdr.ScVal.fromXDR(e.value,"base64")).update_data;
  const prices=new Map(Array.isArray(data)?data:Object.entries(data));
  byRound.set(round,[round,e.ledger,e.ledgerClosedAt,e.id,prices.get("XLM")?.toString()??null,prices.get("USDC")?.toString()??null]);
 }
 const rows=[...byRound.values()].sort((a,b)=>a[0]-b[0]);
 if(rows.length<2)throw new Error("fewer than two Reflector rounds in the RPC window");
 const land=r=>Date.parse(r[2])/1000, delays=rows.map(r=>land(r)-r[0]);
 const gaps={}; for(let i=1;i<rows.length;i++){const g=rows[i][0]-rows[i-1][0];gaps[g]=(gaps[g]||0)+1;}
 // Age is close - round_ts while a round is the latest landed one: from its landing to the ledger before the next.
 const spans=rows.slice(0,-1).map((r,i)=>({round:r[0],from:land(r)-r[0],len:land(rows[i+1])-land(r),next_ledger:rows[i+1][1]}));
 const total=spans.reduce((n,s)=>n+s.len,0), below=a=>spans.reduce((n,s)=>n+Math.min(Math.max(a-s.from,0),s.len),0);
 let lo=0, hi=Math.max(...spans.map(s=>s.from+s.len)); while(lo<hi){const m=Math.floor((lo+hi)/2); if(below(m)>=total/2)hi=m; else lo=m+1;}
 const approx=spans.map(s=>s.from+s.len);
 let max=null; // exact: close(next landing ledger - 1) - round_ts < approx, so stop once approx cannot beat the max
 for(const i of approx.map((_,i)=>i).sort((a,b)=>approx[b]-approx[a])){
  const s=spans[i]; if(max&&approx[i]<=max.age_s)break;
  const ledger=s.next_ledger-1, age=await close(ledger)-s.round;
  if(!max||age>max.age_s)max={age_s:age,round_ts:s.round,round:utc(s.round),ledger,ledger_close:utc(closes.get(ledger))};}
 const span=rows.at(-1)[0]-rows[0][0];
 const summary={rounds:rows.length,first_round:utc(rows[0][0]),last_round:utc(rows.at(-1)[0]),span_s:span,duplicate_events:duplicates,
  delay_s:{min:Math.min(...delays),max:Math.max(...delays)},gaps_s:gaps,time_weighted_age_p50_s:lo,max_age:max,
  rounds_missing_xlm:rows.filter(r=>r[4]===null).map(r=>r[0]),rounds_missing_usdc:rows.filter(r=>r[5]===null).map(r=>r[0])};
 const self_check={rule:"first and last round_ts span at least 24 h",span_s:span,passed:span>=86400};
 if(!self_check.passed)throw new Error("Reflector sample spans "+span+" s, under 24 h");
 return {scan:{oldest_ledger:health.oldestLedger,start_ledger:start,target_ledger:target,pages,events:events.length,limit:1000,
  topics:["REFLECTOR","update","*"]},columns:["round_ts","ledger","ledgerClosedAt","id","xlm","usdc"],rows,summary,self_check};
}

async function readsMode() {
 if(!/^C[A-Z2-7]{55}$/.test(input.v3_copy))throw new Error("testnet.json v3_copy is not set; run testnet-deploy.py v3 first");
 const keys=["inputs","network","reads","code","reflector","blend","xlm_pair"]; absent(keys);
 const captured_at=new Date().toISOString();
 const getNetwork=await rpc("getNetwork");
 if(getNetwork.passphrase!==net.passphrase)throw new Error("Refuse: RPC passphrase is not "+net.passphrase);
 account=new S.Account(input.simulation_source,"0");
 const latest=await rpc("getLatestLedger");
 const network={getNetwork,getVersionInfo:await rpc("getVersionInfo"),getHealth:await rpc("getHealth"),
  getLatestLedger:{sequence:latest.sequence,protocolVersion:latest.protocolVersion,closeTime:utc(Number(latest.closeTime)),id:latest.id}};
 // R7 first: the pair id is an R2 label, and the deviation needs reserves and prices close together.
 const {XLM,USDC}=input.assets, R=input.reflector.id;
 const pairRow=await call(input.soroswap.factory,"get_pair",[address(XLM.sac),address(USDC.sac)]), pair=pairRow.value;
 const [t0,t1,res]=[await call(pair,"token_0"),await call(pair,"token_1"),await call(pair,"get_reserves")];
 const labels={router:input.soroswap.router,factory:input.soroswap.factory,xlm_pair:pair,reflector:R,v3_copy:input.v3_copy,
  ...Object.fromEntries(Object.entries(input.blend.pools).map(([k,v])=>["pool:"+k,v])),
  ...Object.fromEntries(Object.entries(input.blend.backstops).map(([k,v])=>["backstop:"+k,v]))};
 const prices={}; // R3 prices right after the R7 reserves
 for(const s of input.reflector.symbols){const row=await call(R,"lastprice",[feedKey("Other",s)]);
  prices[s]={price:row.value.price.toString(),timestamp:Number(row.value.timestamp),ledger:row.ledger,ledger_close:row.ledger_close,
   age_s:Date.parse(row.ledger_close)/1000-Number(row.value.timestamp)};}
 const contract={}; // R3
 for(const m of ["base","decimals","resolution","cache_size","history_retention_period","admin","version","last_timestamp"])contract[m]=(await call(R,m)).value;
 const code={}; for(const [label,id] of Object.entries(labels))code[label]=await identity(id); // R2
 const reflector={id:R,contract,prices,...await reflectorScan(R)};
 const blend={}; // R5, R6
 for(const [name,pool] of Object.entries(input.blend.pools)){
  const list=(await call(pool,"get_reserve_list")).value, i=list.indexOf(USDC.sac);
  if(i<0)throw new Error(name+" has no USDC reserve");
  const emission=async index=>{const row=await call(pool,"get_reserve_emissions",[u32(index)]), v=row.value;
   return {reserve_token_index:index,value:v,ledger:row.ledger,ledger_close:row.ledger_close,
    status:v===null?"none":Number(v.expiration)<=Date.parse(row.ledger_close)/1000?"expired":"active",expiration:v===null?null:utc(Number(v.expiration))};};
  const reserve=await call(pool,"get_reserve",[address(USDC.sac)]);
  blend[name]={pool,reserve_list:list,usdc_index:i,emissions:{dToken:await emission(2*i),bToken:await emission(2*i+1)},
   get_reserve:{value:reserve.value,ledger:reserve.ledger,ledger_close:reserve.ledger_close,
    last_time:utc(Number(reserve.value.data.last_time)),last_time_minus_close_s:Number(reserve.value.data.last_time)-Date.parse(reserve.ledger_close)/1000}};
 }
 const [r0,r1]=res.value.map(BigInt), usdcFirst=t0.value===USDC.sac;
 if(![t0.value,t1.value].includes(USDC.sac)||![t0.value,t1.value].includes(XLM.sac))throw new Error("pair tokens are not XLM and USDC");
 const [usdc,xlm]=usdcFirst?[r0,r1]:[r1,r0], mid=Number(usdc)/Number(xlm); // both 7 decimals
 const ref=Number(prices.XLM.price)/Number(prices.USDC.price);
 const xlm_pair={pair,token_0:t0.value,token_1:t1.value,reserves:res.value.map(String),usdc_reserve:usdc.toString(),xlm_reserve:xlm.toString(),
  mid_usdc_per_xlm:mid,reflector_xlm_usdc:ref,deviation_bps:(mid/ref-1)*1e4,
  read_ledgers:{get_pair:pairRow.ledger,reserves:res.ledger,reflector_xlm:prices.XLM.ledger,reflector_usdc:prices.USDC.ledger}};
 await call(input.v3_copy,"config"); // the copy's recorded config()
 const m=seqs=>meta(captured_at,seqs), all=reads.map(r=>r.ledger), tip=[latest.sequence];
 const blendLedgers=Object.values(blend).flatMap(b=>[b.emissions.dToken.ledger,b.emissions.bToken.ledger,b.get_reserve.ledger]);
 write({inputs:{...m(tip),file:inputPath,content:input},network:{...m(tip),...network},reads:{...m(all),rows:reads},
  code:{...m(Object.values(code).map(c=>c.ledger)),contracts:code},
  reflector:{...m([reflector.scan.start_ledger,reflector.scan.target_ledger,...Object.values(prices).map(p=>p.ledger)]),...reflector},
  blend:{...m(blendLedgers),pools:blend},xlm_pair:{...m(Object.values(xlm_pair.read_ledgers)),...xlm_pair}});
 console.log(json({code:Object.fromEntries(Object.entries(code).map(([k,v])=>[k,v.wasm_hash||v.executable])),prices,summary:reflector.summary,xlm_pair,
  blend:Object.fromEntries(Object.entries(blend).map(([k,v])=>[k,{dToken:v.emissions.dToken.status,bToken:v.emissions.bToken.status,last_time_minus_close_s:v.get_reserve.last_time_minus_close_s}]))}));
}

function native(v) { // scValToNative, keeping errors and anything it cannot convert
 if(v.type==="scvError")return {error:{type:v.error.type,code:v.error.type==="sceContract"?v.error.contractCode:v.error.code}};
 if(v.type==="scvVec")return v.vec===null?null:v.vec.map(native);
 try{return S.scValToNative(v);}catch{return {xdr:v.toXDR("base64")};}
}
function event(e) {
 return {contract:e.contractId?S.StrKey.encodeContract(Buffer.from(e.contractId.value)):null,type:e.type,
  topics:e.body.v0.topics.map(native),data:native(e.body.v0.data)};
}
async function txMode(hashes) {
 if(!hashes.length)throw new Error("tx mode needs at least one hash"); absent(["raw_get_transaction","decoded_js"]);
 const captured_at=new Date().toISOString(), ev=load();
 if((await rpc("getNetwork")).passphrase!==net.passphrase)throw new Error("Refuse: RPC passphrase is not "+net.passphrase);
 const archived=[ev.direct_proof?.tx,ev.failed_swap?.tx].filter(Boolean), raw={}, decoded={};
 for(const hash of hashes){
  raw[hash]=await rpc("getTransaction",{hash,xdrFormat:"json"});
  if(raw[hash].status==="NOT_FOUND")throw new Error(hash+" is not in the RPC window");
  const tx=archived.find(t=>t.hash===hash); if(!tx)throw new Error(hash+" has no archived result_meta_xdr in "+evidencePath);
  const m=S.xdr.TransactionMeta.fromXDR(tx.result_meta_xdr,"base64"); if(m.type!=="v4")throw new Error(hash+" meta is "+m.type+", not v4");
  const v4=m.v4, ret=v4.sorobanMeta?.returnValue;
  decoded[hash]={source:"archived result_meta_xdr",meta_type:m.type,
   result:S.xdr.TransactionResult.fromXDR(tx.result_xdr,"base64").result.type,
   operations:v4.operations.map(o=>o.events.map(event)),events:v4.events.map(t=>({stage:t.stage,...event(t.event)})),
   diagnostic_events:v4.diagnosticEvents.map(d=>({in_successful_contract_call:d.inSuccessfulContractCall,...event(d.event)})),
   return_value:ret?native(ret):null};
 }
 const m=meta(captured_at,Object.values(raw).map(r=>r.ledger));
 write({raw_get_transaction:{...m,transactions:raw},decoded_js:{...m,transactions:decoded}});
 console.log(json(Object.fromEntries(Object.entries(decoded).map(([h,d])=>[h,{result:d.result,return_value:d.return_value,diagnostic_events:d.diagnostic_events.length}]))));
}

async function activationMode() {
 absent(["mainnet_activation"]); const captured_at=new Date().toISOString();
 if((await rpc("getNetwork")).passphrase!==net.passphrase)throw new Error("Refuse: RPC passphrase is not "+net.passphrase);
 const health=await rpc("getHealth"); let calls=0;
 const header=async seq=>{calls++;const l=(await rpc("getLedgers",{startLedger:seq,pagination:{limit:1},xdrFormat:"json"})).ledgers[0];
  if(l.sequence!==seq)throw new Error("getLedgers returned "+l.sequence+" for "+seq); return {ledger:seq,close:utc(Number(l.ledgerCloseTime)),header:l.headerJson.header};};
 let lo=health.oldestLedger+1, hi=health.latestLedger;
 const [first,last]=[await header(lo),await header(hi)];
 if(last.header.ledger_version<30)throw new Error("mainnet is still on protocol "+last.header.ledger_version+" at ledger "+hi);
 if(first.header.ledger_version>=30)throw new Error("protocol 30 activated before the RPC window ("+lo+")");
 let at=last, before=first;
 while(hi-lo>1){const mid=Math.floor((lo+hi)/2), h=await header(mid); if(h.header.ledger_version>=30){hi=mid;at=h;}else{lo=mid;before=h;}}
 const record={definition:"the activation ledger is the first ledger whose header reports version 30; whether that ledger's own transactions ran under protocol 29 is Open",
  ledger:at.ledger,close:at.close,ledger_version:at.header.ledger_version,upgrades:at.header.scp_value.upgrades,
  previous:{ledger:before.ledger,close:before.close,ledger_version:before.header.ledger_version},
  search:{method:"getLedgers binary search on headerJson.header.ledger_version",low:health.oldestLedger+1,high:health.latestLedger,calls}};
 write({mainnet_activation:{...meta(captured_at,[record.previous.ledger,record.ledger]),...record}});
 console.log(json(record));
}

({reads:readsMode,tx:()=>txMode(rest),activation:activationMode})[mode]().catch(e=>{console.error(e);process.exitCode=1;});
