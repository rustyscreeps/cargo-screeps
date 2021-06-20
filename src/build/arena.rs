use std::{env, ffi::OsStr, fs, io::Write, path::Path};

use failure::{ensure, format_err};
use log::*;

use wasm_pack::command::build::{Build, BuildOptions, Target};

use crate::config::{BuildConfiguration, BuildProfile};

pub fn build(root: &Path, build_config: &BuildConfiguration) -> Result<(), failure::Error> {
    debug!("building");

    debug!("changing directory to {}", root.display());

    env::set_current_dir(&root)?;

    debug!("running wasm-pack build");

    // get the out_name from the build config, or use bindgen's default of the working directory name
    let out_name = match &build_config.out_name {
        Some(v) => v.clone(),
        None => root.file_stem().unwrap().to_str().unwrap().to_string(),
    };

    let (dev, profiling, release) = match &build_config.build_profile {
        Some(profile) => match profile {
            BuildProfile::Dev => (true, false, false),
            BuildProfile::Profiling => (false, true, false),
            BuildProfile::Release => (false, false, true),
        },
        None => (false, false, true),
    };

    let options = BuildOptions {
        path: build_config.path.clone(),
        target: Target::Web,
        out_dir: "pkg".to_string(),
        out_name: Some(out_name.clone()),
        extra_options: build_config.extra_options.clone(),
        dev,
        release,
        profiling,
        ..Default::default()
    };

    Build::try_from_opts(options).and_then(|mut b| b.run())?;

    debug!("finished executing wasm-pack build");

    let target_dir = build_config
        .path
        .as_ref()
        .map(|p| root.join(p))
        .unwrap_or_else(|| root.into())
        .join("pkg");

    let mut generated_js = None;
    for r in fs::read_dir(&target_dir)? {
        let entry = r?;
        let file_name = entry.file_name();
        let file_name = Path::new(&file_name);
        match file_name.extension().and_then(OsStr::to_str) {
            Some("js") => {
                ensure!(
                    generated_js.is_none(),
                    "error: multiple js files found in {}",
                    target_dir.display()
                );
                generated_js = Some(entry.path());
            }
            _ => {}
        }
    }
    let generated_js = generated_js
        .ok_or_else(|| format_err!("error: no js files found in {}", target_dir.display()))?;

    let mut generated_wasm = None;
    for r in fs::read_dir(&target_dir)? {
        let entry = r?;
        let file_name = entry.file_name();
        let file_name = Path::new(&file_name);
        match file_name.extension().and_then(OsStr::to_str) {
            Some("wasm") => {
                ensure!(
                    generated_wasm.is_none(),
                    "error: multiple wasm files found in {}",
                    target_dir.display()
                );
                generated_wasm = Some(entry.path());
            }
            _ => {}
        }
    }
    let generated_wasm = generated_wasm
        .ok_or_else(|| format_err!("error: no wasm files found in {}", target_dir.display()))?;

    debug!("loading wasm file");
    let generated_wasm_contents = fs::read(&generated_wasm)?;
    let generated_wasm_b64 = base64::encode(&generated_wasm_contents);

    let folded_wasm_b64 = generated_wasm_b64.chars().enumerate()
        .fold(String::new(), |acc, (i, c)| {
            if i % 100 == 0 {
                format!("{}' +\n'{}", acc, c)
            } else {
                format!("{}{}", acc, c)
            }
        });

    let generated_wasm_rename_to = generated_wasm.with_extension("wasmorig");
    fs::rename(&generated_wasm, &generated_wasm_rename_to)?;

    debug!("processing js file");

    let generated_js_contents = fs::read_to_string(&generated_js)?;

    let generated_js_rename_to = generated_js.with_extension("jsorig");
    fs::rename(&generated_js, &generated_js_rename_to)?;

    let processed_js = process_js(&generated_js, &folded_wasm_b64, &generated_js_contents)?;

    let destination_mjs = generated_js.with_file_name("main.mjs");

    debug!("writing processed mjs with embedded wasm to {}", destination_mjs.display());

    let mut output_handle = fs::File::create(destination_mjs)?;
    output_handle.write_all(processed_js.as_bytes())?;
    output_handle.flush()?;

    Ok(())
}

fn process_js(file_name: &Path, wasm_data_b64: &str, input: &str) -> Result<String, failure::Error> {
    // add polyfills for TextEncoder/TextDecoder as well as base64 conversion,
    // replace loader functions, and embed the base64-encoded wasm module in the mjs file
    let bindgen_output_regex = regex::Regex::new(&format!(
        "(?s)(.+)\n{}\n.+{}\n.+({}.+){}.+{}.*",
        regex::escape("async function load(module, imports) {"),
        regex::escape("async function init(input) {"),
        regex::escape("const imports = {};"),
        regex::escape("if (typeof input === 'string'"),
        regex::escape("export default init;"),
    ))
    .expect("expected pre-set regex to succeed");

    let captures = bindgen_output_regex.captures(input).ok_or_else(|| {
        format_err!(
            "'wasm-pack' generated unexpected JS output! This means it's updated without \
             'cargo screeps' also having updated. Please report this issue to \
             https://github.com/rustyscreeps/cargo-screeps/issues and include \
             the first ~30 lines of {}",
            file_name.display(),
        )
    })?;

    // CC-0 TextEncoder/TextDecoder polyfill from https://github.com/anonyco/FastestSmallestTextEncoderDecoder
    // Apache 2.0 base64 encode/decode polyfill from https://github.com/davidchambers/Base64.js
    Ok(format!(
        r#"'use strict';(function(r){{function x(){{}}function y(){{}}var z=String.fromCharCode,v={{}}.toString,A=v.call(r.SharedArrayBuffer),B=v(),q=r.Uint8Array,t=q||Array,w=q?ArrayBuffer:t,C=w.isView||function(g){{return g&&"length"in g}},D=v.call(w.prototype);w=y.prototype;var E=r.TextEncoder,a=new (q?Uint16Array:t)(32);x.prototype.decode=function(g){{if(!C(g)){{var l=v.call(g);if(l!==D&&l!==A&&l!==B)throw TypeError("Failed to execute 'decode' on 'TextDecoder': The provided value is not of type '(ArrayBuffer or ArrayBufferView)'");
g=q?new t(g):g||[]}}for(var f=l="",b=0,c=g.length|0,u=c-32|0,e,d,h=0,p=0,m,k=0,n=-1;b<c;){{for(e=b<=u?32:c-b|0;k<e;b=b+1|0,k=k+1|0){{d=g[b]&255;switch(d>>4){{case 15:m=g[b=b+1|0]&255;if(2!==m>>6||247<d){{b=b-1|0;break}}h=(d&7)<<6|m&63;p=5;d=256;case 14:m=g[b=b+1|0]&255,h<<=6,h|=(d&15)<<6|m&63,p=2===m>>6?p+4|0:24,d=d+256&768;case 13:case 12:m=g[b=b+1|0]&255,h<<=6,h|=(d&31)<<6|m&63,p=p+7|0,b<c&&2===m>>6&&h>>p&&1114112>h?(d=h,h=h-65536|0,0<=h&&(n=(h>>10)+55296|0,d=(h&1023)+56320|0,31>k?(a[k]=n,k=k+1|0,n=-1):
(m=n,n=d,d=m))):(d>>=8,b=b-d-1|0,d=65533),h=p=0,e=b<=u?32:c-b|0;default:a[k]=d;continue;case 11:case 10:case 9:case 8:}}a[k]=65533}}f+=z(a[0],a[1],a[2],a[3],a[4],a[5],a[6],a[7],a[8],a[9],a[10],a[11],a[12],a[13],a[14],a[15],a[16],a[17],a[18],a[19],a[20],a[21],a[22],a[23],a[24],a[25],a[26],a[27],a[28],a[29],a[30],a[31]);32>k&&(f=f.slice(0,k-32|0));if(b<c){{if(a[0]=n,k=~n>>>31,n=-1,f.length<l.length)continue}}else-1!==n&&(f+=z(n));l+=f;f=""}}return l}};w.encode=function(g){{g=void 0===g?"":""+g;var l=g.length|
0,f=new t((l<<1)+8|0),b,c=0,u=!q;for(b=0;b<l;b=b+1|0,c=c+1|0){{var e=g.charCodeAt(b)|0;if(127>=e)f[c]=e;else{{if(2047>=e)f[c]=192|e>>6;else{{a:{{if(55296<=e)if(56319>=e){{var d=g.charCodeAt(b=b+1|0)|0;if(56320<=d&&57343>=d){{e=(e<<10)+d-56613888|0;if(65535<e){{f[c]=240|e>>18;f[c=c+1|0]=128|e>>12&63;f[c=c+1|0]=128|e>>6&63;f[c=c+1|0]=128|e&63;continue}}break a}}e=65533}}else 57343>=e&&(e=65533);!u&&b<<1<c&&b<<1<(c-7|0)&&(u=!0,d=new t(3*l),d.set(f),f=d)}}f[c]=224|e>>12;f[c=c+1|0]=128|e>>6&63}}f[c=c+1|0]=128|e&63}}}}return q?
f.subarray(0,c):f.slice(0,c)}};E||(r.TextDecoder=x,r.TextEncoder=y)}})(""+void 0==typeof global?""+void 0==typeof self?this:self:global);

!function(e){{"use strict";if("object"==typeof exports&&null!=exports&&"number"!=typeof exports.nodeType)module.exports=e();else if("function"==typeof define&&null!=define.amd)define([],e);else{{var t=e(),o=global;"function"!=typeof o.btoa&&(o.btoa=t.btoa),"function"!=typeof o.atob&&(o.atob=t.atob)}}}}(function(){{"use strict";var f="ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";function c(e){{this.message=e}}return(c.prototype=new Error).name="InvalidCharacterError",{{btoa:function(e){{for(var t,o,r=String(e),n=0,a=f,i="";r.charAt(0|n)||(a="=",n%1);i+=a.charAt(63&t>>8-n%1*8)){{if(255<(o=r.charCodeAt(n+=.75)))throw new c("'btoa' failed: The string to be encoded contains characters outside of the Latin1 range.");t=t<<8|o}}return i}},atob:function(e){{var t=String(e).replace(/[=]+$/,"");if(t.length%4==1)throw new c("'atob' failed: The string to be decoded is not correctly encoded.");for(var o,r,n=0,a=0,i="";r=t.charAt(a++);~r&&(o=n%4?64*o+r:r,n++%4)&&(i+=String.fromCharCode(255&o>>(-2*n&6))))r=f.indexOf(r);return i}}}}}});
{}

async function load(buffer, imports) {{
    let wasm_module = new WebAssembly.Module(buffer);
    return new WebAssembly.Instance(wasm_module, imports);
}}

async function init() {{
    {}

    const instance = await load(wasm_bytes, imports);
    wasm = instance.exports;
}}

let wasm_b64 = '{}';

let wasm_decoded = atob(wasm_b64);

wasm_b64 = null;

var len = wasm_decoded.length;
var wasm_bytes = new Uint8Array(len);
for (var i = 0; i < len; i++) {{
    wasm_bytes[i] = wasm_decoded.charCodeAt(i);
}}

wasm_decoded = null;

await init();

wasm_bytes = null;
"#,
        &captures[1], &captures[2], wasm_data_b64
    ))
}
