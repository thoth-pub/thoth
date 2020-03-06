use crate::client::work_query::WorkQueryWork;
use crate::errors::Result;

pub fn generate_onix_3(work: WorkQueryWork) -> Result<()> {
    println!("{:#?}", work);
    let doi: String = work.doi.unwrap();
    println!("{:#?}", doi);
    Ok(())
}
