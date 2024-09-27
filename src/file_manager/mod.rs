  use std::fs::OpenOptions;
  use std::io::{ BufReader, BufRead, Result };

  pub fn lock(file_name : &String, begin : &usize, end : &usize) -> Result<bool>  {
    let file = OpenOptions::new().read(true).open(file_name)?;
    let line_count = BufReader::new(file).lines().count();
    
    if line_count < *end {
      println!("Requested lock goes beyond file's line count");
      return Ok(false)
    }
    
    Ok(true)
  }