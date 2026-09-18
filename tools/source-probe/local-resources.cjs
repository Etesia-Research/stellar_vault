// Measure unsigned reads against the isolated network only.
const fs=require("node:fs");
const S=require("@stellar/stellar-sdk");
const manifest=JSON.parse(fs.readFileSync(process.argv[2]));
const url="http://127.0.0.1:8003";
async function rpc(method,params={}) {
 const r=await fetch(url,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({jsonrpc:"2.0",id:1,method,params})});
 const p=await r.json();if(p.error)throw new Error(JSON.stringify(p.error));return p.result;
}
(async()=>{
 const network=await rpc("getNetwork");
 if(network.passphrase!=="Standalone Network ; September 2026")throw new Error("Not isolated local network");
 const calls=[];
 for(const [id,method,args] of [[manifest.vault,"total_assets",[]],[manifest.provider,"mark",[new S.Address(manifest.provider_config.sources[0].asset).toScVal()]]]) {
  const tx=new S.TransactionBuilder(new S.Account(manifest.config.admin,"0"),{fee:"100",networkPassphrase:network.passphrase}).addOperation(new S.Contract(id).call(method,...args)).setTimeout(60).build();
  const r=await rpc("simulateTransaction",{transaction:tx.toXDR()});if(r.error)throw new Error(r.error);
  const data=S.xdr.SorobanTransactionData.fromXDR(r.transactionData,"base64").resources();
  calls.push({id,method,ledger:r.latestLedger,value:S.scValToNative(S.xdr.ScVal.fromXDR(r.results[0].xdr,"base64")),instructions:data.instructions(),read_bytes:data.readBytes(),write_bytes:data.writeBytes(),read_entries:data.footprint().readOnly().length,write_entries:data.footprint().readWrite().length,min_resource_fee:r.minResourceFee,cost:r.cost||null});
 }
 fs.writeFileSync(process.argv[3],JSON.stringify({captured_at:new Date().toISOString(),scope:"local protocol 22 simulation; instructions include RPC margin; successful under 40 MiB configured memory",calls},(_,v)=>typeof v==="bigint"?v.toString():v,2)+"\n");
})().catch(e=>{console.error(e);process.exitCode=1;});
