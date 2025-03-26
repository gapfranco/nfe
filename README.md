# nfe

Processador de XML de notas fiscais eletronicas. 

### Objetivos

Ler arquivos XML e gravar numa tabela num banco de dados Postgres.

```aiignore
Usage: nfe [OPTIONS] --xml <ARQUIVO>

Options:
  -d, --database <DATABASE>  Database connection
  -x, --xml <ARQUIVO>        Arquivo XML
  -g, --grupo <GRUPO>        ID Grupo
  -e, --empresa <EMPRESA>    ID Empresa
  -f, --filial <FILAL>       ID Filial
  -h, --help                 Print help
  -V, --version              Print version

```
### Detalhamento

Le e processa um arquivo XML, conecta a um banco de dados *Postres* e grava registros
numa tabela desse banco de dados, usando os parametros numérios **grupo**, **empresa** e **filial**
para identificar cada registro.

