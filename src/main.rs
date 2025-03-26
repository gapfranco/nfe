use clap::Parser;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Importador de XML
struct Args {
    /// Database connection
    #[arg(short('d'), long("database"))]
    database: Option<String>,

    /// Arquivo XML
    #[arg(short('x'), long("xml"))]
    arquivo: String,

    /// ID Grupo
    #[arg(short('g'), long("grupo"))]
    grupo: Option<i32>,

    /// ID Empresa
    #[arg(short('e'), long("empresa"))]
    empresa: Option<i32>,

    /// ID Filial
    #[arg(short('f'), long("filial"))]
    filal: Option<i32>,

}

fn main() {
    let args = Args::parse();

    let file = File::open(&args.arquivo).expect("Falha ao abrir o arquivo");
    let mut xml_reader = Reader::from_reader(BufReader::new(file));
    xml_reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    loop {
        match xml_reader.read_event_into(&mut buf) { // Usando read_event_into
            Ok(Event::Start(e)) => {
                println!("Tag: {}", String::from_utf8_lossy(e.name().as_ref()));
                for attr in e.attributes() {
                    let attr = attr.unwrap();
                    println!(
                        "  Atributo: {} = {}",
                        String::from_utf8_lossy(attr.key.as_ref()),
                        String::from_utf8_lossy(&*attr.value)
                    );
                }
            }
            Ok(Event::Text(e)) => {
                println!("  Valor: {}", String::from_utf8_lossy(&e));
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Erro ao ler XML: {}", e),
            _ => (),
        }
        buf.clear(); // Limpa o buffer para o próximo evento
    }

}
