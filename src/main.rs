use std::collections::HashMap;
use walkdir::WalkDir;
use std::path::Path;
use clap::Parser;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::BufReader;
use sqlx::{postgres::PgPoolOptions, Error, PgPool, Row}; //, Postgres}; //, Row};
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
    let mut item  = "".to_string();
    let mut nivel = 0;

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let mut key = String::from_utf8_lossy(e.name().as_ref()).to_string();
                nivel += 1;
                if nivel > 3 {
                    if key == "det" {
                        let mut tags2 : Vec<String> = vec![];
                        tags2.push("det".to_string());
                        for attr in e.attributes() {
                            let attr = attr.unwrap();
                            tags2.push(format!("{}",String::from_utf8_lossy(&*attr.value)));
                        }
                        item = tags2.join("_");
                    }
                    if item != "" && key != "det"  {
                        key = format!("{}_{}", item, key);
                    }
                    tags.push(key);
                }
            }
            Ok(Event::Text(e)) => {
                let value = String::from_utf8_lossy(&e).to_string();
                let key2 = tags.join("_");
                retorno.insert(key2, value);
            }
            Ok(Event::End(e)) => {
                let value = String::from_utf8_lossy(&e).to_string();
                if value == "det" {
                    item = "".to_string();
                }
                tags.pop();
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

    let database_url = args.database.clone().unwrap();
    let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url).await?;

    if caminho.is_file() {
        let resultado = processar_arquivo(caminho).await?;
        mostra_resultado(&resultado);

        if args.database.is_some() {
            // println!("Processando no BD {:?}", pool);
            processar_db(resultado, &pool).await.unwrap();
        }
    } else if caminho.is_dir() {
        for entry in WalkDir::new(caminho) {
            let entry = entry?;
            if entry.path().is_file() && entry.path().extension().unwrap_or_default() == "xml" {
                let resultado = processar_arquivo(entry.path()).await?;
                // println!("Resultados para o arquivo {:?}:\n{:#?}", entry.path(), resultado);
                mostra_resultado(&resultado);

                if args.database.is_some() {
                    // println!("Processando no BD {:?}", pool)
                    processar_db(resultado, &pool).await.unwrap();
                }
            }
        }
    } else {
        println!("O caminho fornecido não é um arquivo nem uma pasta.");
    }

    Ok(())
}

async fn processar_db(resultado: HashMap<String, String>, pool: &PgPool)  -> Result<(), Error> {
    let ch_nfe = resultado.get("protNFe_infProt_chNFe").unwrap();
    println!("Em processar_db {}", ch_nfe);

    let cnpj = resultado.get("dest_CNPJ").unwrap();
    println!("CNPJ: {}", cnpj);

    let row = sqlx::query(
        "SELECT tenant_id, tbsg1301codemp, tbsg1301local, tbsg1301deslocal
            FROM tbsg1301_cad_grupo_empr_local where tbsg1301cgccpf = $1")
        .bind(cnpj)
        .fetch_one(pool)
        .await;
    if let Ok(row) = row {
        let nome = row.get::<String, &str>("tbsg1301desloca\
        l");
        let id_grupo = row.get::<i32, &str>("tenant_id");
        let codemp = row.get::<i32, &str>("tbsg1301codemp");
        let local = row.get::<i32, &str>("tbsg1301local");
        println!("NOME: {} GRUPO: {} CODEMP: {} LOCAL: {}", &nome, id_grupo, codemp, local);
        println!("{:#?}", row);

        // # insert tabela(id_grupo, codemp, local, cpf ...)

    } else {
        println!("Registro não encontrado, continuando o processamento.");
    }

    Ok(())
}

fn mostra_resultado(resultado: &HashMap<String, String>) {
    let mut chaves: Vec<&String> = resultado.keys().collect();
    chaves.sort(); // Ordena as chaves alfabeticamente
    println!("\nResultado ------------------------------------------\n");
    for chave in chaves {
        let valor = resultado.get(chave).unwrap(); // Obtem o valor correspondente
        println!("Chave: {}, Valor: {}", chave, valor);
    }
}
