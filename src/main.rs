use anyhow::Result;

mod bencode;

fn main() -> Result<()> {
    let file = std::fs::read("file1.txt.torrent")?;
    let mut parser = bencode::Parser::new(file);
    let data = parser.parse()?;
    println!("{:#?}", data);

    Ok(())
}
