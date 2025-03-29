use std::collections::HashMap;
use walkdir::WalkDir;
use std::path::Path;
use clap::Parser;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::BufReader;
use sqlx::{postgres::PgPoolOptions}; //, Row};
use anyhow;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Importador de XML
struct Args {
    /// Database connection
    #[arg(short('d'), long("database"))]
    database: Option<String>,

    /// Arquivo XML ou pasta co esses aqruivos
    #[arg(short('p'), long("path"))]
    arquivo: String,

}

//     // database_url
//     // "postgres://hover:password@localhost:5432/hover-pro";
//     // "postgresql://postgres:kZ6KY63xcyfQgrAa@db.hiibsdfabwmiqvrtwplx.supabase.co:5432/postgres";

async fn processar_arquivo(caminho: &Path) -> anyhow::Result<HashMap<String, String>> {
    let file = File::open(caminho).expect("Falha ao abrir o arquivo XML");
    let mut xml_reader = Reader::from_reader(BufReader::new(file));
    xml_reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut tags : Vec<String> = vec![];
    let mut retorno: HashMap<String, String> = HashMap::new() ;

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let key = String::from_utf8_lossy(e.name().as_ref()).to_string();
                tags.push(key);
            }
            Ok(Event::Text(e)) => {
                let value = String::from_utf8_lossy(&e).to_string();
                let key2 = tags.join("/");
                retorno.insert(key2, value);
                tags.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Erro ao ler XML: {}", e),
            _ => (),
        }
        buf.clear();
    }


    Ok(retorno)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let caminho = Path::new(&args.arquivo);

    // Cria um pool de conexões com o banco de dados
    // if args.database.is_some() {
    let database_url = args.database.clone().unwrap();
    // let pool: Option<sqlx::Pool<sqlx::Postgres>> = None;

    let pool = if args.database.is_some() {
        Some(PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url).await?
        )
    } else {
        None
    };

    if caminho.is_file() {
        let resultado = processar_arquivo(caminho).await?;
        println!("{:#?}", resultado);
        if args.database.is_some() {
            println!("Processando no BD {:?}", pool)
        }
    } else if caminho.is_dir() {
        for entry in WalkDir::new(caminho) {
            let entry = entry?;
            if entry.path().is_file() && entry.path().extension().unwrap_or_default() == "xml" {
                let resultado = processar_arquivo(entry.path()).await?;
                println!("Resultados para o arquivo {:?}:\n{:#?}", entry.path(), resultado);
                if args.database.is_some() {
                    println!("Processando no BD {:?}", pool)
                }
            }
        }
    } else {
        println!("O caminho fornecido não é um arquivo nem uma pasta.");
    }

    Ok(())
}

// #[tokio::main]
// async fn main() ->anyhow::Result<()> {
//     let args = Args::parse();
//
//     // if let Ok(entries) = fs::read_dir(".") {
//     //     for entry in entries {
//     //         if let Ok(entry) = entry {
//     //             // Here, `entry` is a `DirEntry`.
//     //             if let Ok(file_type) = entry.file_type() {
//     //                 // Now let's show our entry's file type!
//     //                 println!("{:?}: {:?}", entry.path(), file_type);
//     //             } else {
//     //                 println!("Couldn't get file type for {:?}", entry.path());
//     //             }
//     //         }
//     //     }
//     // }
//
//     // database_url
//     // "postgres://hover:password@localhost:5432/hover-pro";
//     // "postgresql://postgres:kZ6KY63xcyfQgrAa@db.hiibsdfabwmiqvrtwplx.supabase.co:5432/postgres";
//
//     let file = File::open(&args.arquivo).expect("Falha ao abrir o arquivo XML");
//     let mut xml_reader = Reader::from_reader(BufReader::new(file));
//     xml_reader.config_mut().trim_text(true);
//
//     let mut buf = Vec::new();
//     // let mut key = String::new();
//     let mut tags : Vec<String> = vec![];
//     let mut retorno: HashMap<String, String> = HashMap::new() ;
//
//     loop {
//         match xml_reader.read_event_into(&mut buf) { // Usando read_event_into
//             Ok(Event::Start(e)) => {
//                 let key = String::from_utf8_lossy(e.name().as_ref()).to_string();
//                 tags.push(key.clone());
//                 // println!("Tag: {}", key);
//                 // for attr in e.attributes() {
//                 //     let attr = attr.unwrap();
//                 //     println!(
//                 //         "  Atributo: {} = {}",
//                 //         String::from_utf8_lossy(attr.key.as_ref()),
//                 //         String::from_utf8_lossy(&*attr.value)
//                 //     );
//                 // }
//             }
//             Ok(Event::Text(e)) => {
//                 let value = String::from_utf8_lossy(&e).to_string();
//                 let key2 = tags.join("/");
//                 retorno.insert(key2.clone(), value.clone());
//                 tags.clear();
//                 // println!("  Valor: {}", value);
//             }
//             Ok(Event::Eof) => break,
//             Err(e) => panic!("Erro ao ler XML: {}", e),
//             _ => (),
//         }
//         buf.clear(); // Limpa o buffer para o próximo evento
//     }
//
//     // Cria um pool de conexões com o banco de dados
//     if args.database.is_some() {
//         let database_url = args.database.unwrap();
//         let pool = PgPoolOptions::new()
//             .max_connections(5)
//             .connect(&database_url).await?;
//         let rows: Vec<sqlx::postgres::PgRow> = sqlx::query("SELECT id, tbca07codemp, tbca07desemp FROM tbca07_cad_empresa")
//             .fetch_all(&pool)
//             .await?;
//         println!("Registros no banco de dados:");
//         for row in rows {
//             let id: i64 = row.try_get("id")?;
//             let codigo: i32 = row.try_get("tbca07codemp")?;
//             let nome: String = row.try_get("tbca07desemp")?;
//             println!("ID: {}, Codigo: {}, Nome: {}", id, codigo, nome);
//         }
//     }
//
//     println!("{:#?}", retorno);
//     Ok(())
// }
