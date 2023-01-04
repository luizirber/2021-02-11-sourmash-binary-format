use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use log::info;
use serde::ser::Serialize;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(parse(from_os_str))]
    sigpath: PathBuf,

    #[structopt(parse(from_os_str), short = "o", long = "output")]
    output_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let Cli {
        sigpath,
        output_path,
    } = Cli::from_args();

    let signature = sourmash::signature::Signature::from_path(&sigpath)?;

    let outdir: PathBuf = if let Some(p) = output_path {
        p
    } else {
        let mut path = PathBuf::new();
        path.push("outputs");
        path
    };
    std::fs::create_dir_all(&outdir)?;

    // Avro
    let sig_schema = r#"
{
    "name": "Signature",
    "type":"record",
    "fields":[
       { "name": "class", "type": "string"},
       { "name": "email", "type": "string"},
       { "name": "hash_function", "type": "string"},
       { "name": "filename", "type": "string"},
       { "name": "name", "type": "string"},
       { "name": "license", "type": "string"},
       { "name": "version", "type": "float" },

       { "name": "signatures",
         "type": {
            "type": "array",
            "items": {

           "name": "MinHash",
           "type": "record",
           "fields":[
             { "name": "num", "type": "int" },
             { "name": "ksize", "type": "int" },
             { "name": "seed", "type": "int" },
             { "name": "max_hash", "type": { "name": "ulong", "type": "fixed", "size": 8 } },
             { "name": "md5sum", "type": "string" },
             { "name":"mins",
               "type": {
                  "type": "array",  
                   "items":{
                       "name":"hash",
                       "type":"fixed",
                       "size": 8
                   }
                }
             },
             { "name":"abunds",
               "type": {
                  "type": "array",  
                  "items":{
                     "name":"abund",
                     "type":"int"
                   }
                }
             }
           ]
         }
       }
       }
 
       ]
}
"#;

    let schema = avro_rs::Schema::parse_list(&[sig_schema])?;

    let mut writer = avro_rs::Writer::with_codec(&schema[0], Vec::new(), avro_rs::Codec::Deflate);

    info!("avro: encoding");
    writer.append_ser(&signature)?;
    let encoded = writer.into_inner()?;
    dbg!(encoded);
    info!("avro: done");

    info!("flexbuffer: encoding");
    //let mut s = flexbuffers::FlexbufferSerializer::new();
    //signature.serialize(&mut s)?;
    let data = flexbuffers::to_vec(&signature)?;
    info!("flexbuffer: saving");
    write_output(&outdir, &sigpath, "flexbuffers", data)?;
    info!("flexbuffer: done");

    //let r = flexbuffers::Reader::get_root(s.view()).unwrap();

    // bincode
    info!("bincode: encoding");
    let data: Vec<u8> = bincode::serialize(&signature)?;
    info!("bincode: saving");
    write_output(&outdir, &sigpath, "bincode", data)?;
    info!("bincode: done");

    // cbor
    info!("cbor: encoding");
    let mut data = vec![];
    serde_cbor::to_writer(&mut data, &signature)?;
    info!("cbor: saving");
    write_output(&outdir, &sigpath, "cbor", data)?;
    info!("cbor: done");

    // postcard
    info!("postcard: encoding");
    let data = postcard::to_allocvec(&signature)?;
    info!("postcard: saving");
    write_output(&outdir, &sigpath, "postcard", data)?;
    info!("postcard: done");

    // msgpack
    info!("msgpack: encoding");
    let data = rmp_serde::to_vec(&signature)?;
    info!("msgpack: saving");
    write_output(&outdir, &sigpath, "msgpack", data)?;
    info!("msgpack: done");

    Ok(())
}

fn write_output(outdir: &PathBuf, sigpath: &PathBuf, extension: &str, data: Vec<u8>) -> Result<()> {
    let mut outpath = outdir.clone();
    outpath.push(sigpath.file_name().unwrap());
    outpath.set_extension(extension);

    let mut out = BufWriter::new(File::create(&outpath)?);
    out.write_all(&data[..])?;

    outpath.set_extension(format!("{}.gz", extension));
    let mut out = niffler::to_path(
        &outpath,
        niffler::compression::Format::Gzip,
        niffler::Level::Nine,
    )?;
    out.write_all(&data[..])?;

    Ok(())
}
