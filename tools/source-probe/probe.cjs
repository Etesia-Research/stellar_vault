// Unsigned read-only mainnet discovery. No signing or submission code.
const fs = require("node:fs");
const crypto = require("node:crypto");
const S = require("@stellar/stellar-sdk");
const rpc = new S.rpc.Server("https://mainnet.sorobanrpc.com");
const source = "GCLA2E3LQDPAPJLHYDMB5R65ASGLNXWGJCX4TX7XA75C7VTJ7Y2OTZXA";
const stellar = "CALI2BYU2JE6WVRUFYTS6MSBNEHGJ35P4AVCZYF3B6QOE3QKOB2PLE6M";
const external = "CAFJZQWSED6YAWZU3GWRTOCNPPCGBN32L7QV43XX5LZLFTK6JLN34DLN";
const factory = "CA4HEQTL2WPEUYKYKCDOHCDNIV4QHNJ7EL4J4NQ6VADP7SYHVRYZ7AW2";
const assets = {
  USDC: ["USDC", "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN"], XLM: null,
  AQUA: ["AQUA", "GBNZILSTVQZ4R7IKQDGHYGY2QXL5QOFJYQMXPKWRRM5PAV7Y4M67AQUA"],
  SHX: ["SHX", "GDSTRSHXHGJ7ZIVRBXEYE5Q74XUVCUSEKEBR7UCHEUUEK72N7I7KJ6JH"],
  USTRY: ["USTRY", "GCRYUGD5NVARGXT56XEZI5CIFCQETYHAPQQTHO2O3IQZTHDH4LATMYWC"],
};
const ids = Object.fromEntries(Object.entries(assets).map(([k,v]) => [k,(v ? new S.Asset(...v) : S.Asset.native()).contractId(S.Networks.PUBLIC)]));
const report = {captured_at:new Date().toISOString(),network:S.Networks.PUBLIC,status:"research-only; no activation",baseline:"8aa929092435ab82fccb8ae24f00bba9e4cc1387",assets:{},feeds:{},pools:{},calls:[]};
const json = value => JSON.stringify(value, (_,v) => typeof v === "bigint" ? v.toString() : v, 2);
let account;
async function call(id, method, args=[]) {
 const tx=new S.TransactionBuilder(account,{fee:"100",networkPassphrase:S.Networks.PUBLIC}).addOperation(new S.Contract(id).call(method,...args)).setTimeout(60).build();
 const response=await fetch("https://mainnet.sorobanrpc.com",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({jsonrpc:"2.0",id:1,method:"simulateTransaction",params:{transaction:tx.toXDR()}})});
 if(!response.ok)throw new Error("RPC HTTP "+response.status);
 const payload=await response.json(); if(payload.error)throw new Error(JSON.stringify(payload.error));
 const result=payload.result;
 const row={captured_at:new Date().toISOString(),id,method,args:args.map(a=>a.toXDR("base64")),ledger:result.latestLedger};
 if(result.error){row.error=result.error;report.calls.push(row);throw new Error(result.error);}
 row.value=S.scValToNative(S.xdr.ScVal.fromXDR(result.results[0].xdr,"base64"));
 row.resources=result.cost; row.restore_required=Boolean(result.restorePreamble);
 report.calls.push(row); return row.value;
}
async function attempt(fn){try{return {status:"verified",value:await fn()};}catch(e){return {status:"unverified",error:String(e.message).slice(0,1200)};}}
async function identity(id){
 const r=await rpc.getLedgerEntries(new S.Contract(id).getFootprint());
 const inst=r.entries[0].val.contractData().val().instance();
 if(inst.executable().switch().name!=="contractExecutableWasm")return {ledger:r.latestLedger,executable:"stellar-asset"};
 const hash=inst.executable().wasmHash();
 const code=await rpc.getLedgerEntries(S.xdr.LedgerKey.contractCode(new S.xdr.LedgerKeyContractCode({hash})));
 const wasm=code.entries[0].val.contractCode().code();
 fs.mkdirSync("artifacts/local/upstream",{recursive:true});
 fs.writeFileSync("artifacts/local/upstream/"+hash.toString("hex")+".wasm",wasm);
 const module=new WebAssembly.Module(wasm);
 const specs=WebAssembly.Module.customSections(module,"contractspecv0");
 return {ledger:r.latestLedger,wasm_hash:hash.toString("hex"),sha256:crypto.createHash("sha256").update(wasm).digest("hex"),bytes:wasm.length,spec_bytes:specs.reduce((n,v)=>n+v.byteLength,0)};
}
const feedKey=(kind,id)=>S.xdr.ScVal.scvVec([S.nativeToScVal(kind,{type:"symbol"}),kind==="Stellar"?new S.Address(id).toScVal():S.nativeToScVal(id,{type:"symbol"})]);
(async()=>{
 account=new S.Account(source,"0"); report.network_info=await rpc.getNetwork();
 for(const [label,id] of [["stellar",stellar],["external",external]]){
  report.feeds[label]={id,identity:await attempt(()=>identity(id))};
  for(const method of ["base","decimals","resolution","assets","history_retention_period","admin","version"])report.feeds[label][method]=await attempt(()=>call(id,method));
 }
 for(const [symbol,id] of Object.entries(ids)){
  report.assets[symbol]={id,issuer:assets[symbol],identity:await attempt(()=>identity(id)),decimals:await attempt(()=>call(id,"decimals")),reflector:await attempt(()=>call(stellar,"lastprice",[feedKey("Stellar",id)]))};
  if(symbol==="USDC")continue;
  report.pools[symbol]=await attempt(async()=>{
   const pool=await call(factory,"get_pair",[new S.Address(id).toScVal(),new S.Address(ids.USDC).toScVal()]);
   return {id:pool,identity:await attempt(()=>identity(pool)),token0:await call(pool,"token_0"),token1:await call(pool,"token_1"),reserves:await call(pool,"get_reserves")};
  });
 }
 for(const symbol of ["BTC","ETH","USDC"])report.assets[symbol+"_reference"]={wrapper_status:"unverified",reflector:await attempt(()=>call(external,"lastprice",[feedKey("Other",symbol)]))};
 report.completed_at=new Date().toISOString();fs.writeFileSync(process.argv[2]||"probe.json",json(report)+"\n");
 console.log(json({captured_at:report.captured_at,assets:report.assets,pools:report.pools}));
})().catch(e=>{console.error(e);process.exitCode=1;});
