use derive_new::new;
pub use sqlite::Error as SqliteError;
use sqlite::{Connection, Value};

pub struct Database {
    connection: Connection
}

#[derive(Debug, new)]
pub struct EtfData {
    pub id: String,
    pub isin: String,
    pub name: String,
    pub proportion: f64,
    pub cumulative: i64,
}

impl Database {
    pub fn new(file_path: &str) -> Result<Database, SqliteError> {
        let connection = sqlite::open(file_path)?;

        let query = "
            CREATE TABLE IF NOT EXISTS etf (id TEXT PRIMARY KEY, isin TEXT, name TEXT, proportion FLOAT, cumulative INTEGER);
        ";
        connection.execute(query)?;
    
        Ok(Database { connection })
    }

    pub fn add_etf(&self, etf: EtfData) -> Result<(), SqliteError> {
        let query = "
            INSERT INTO etf (id, isin, name, proportion, cumulative) 
            VALUES (:id, :isin, :name, :proportion, :cumulative);
        ";

        let mut statement = self.connection.prepare(query)?;

        statement.bind::<&[(_, Value)]>(&[
            (":id", etf.id.into()),
            (":isin", etf.isin.into()),
            (":name", etf.name.into()),
            (":proportion", etf.proportion.into()),
            (":cumulative", etf.cumulative.into()),
        ])?;

        statement.next()?;

        Ok(())
    }
    
    pub fn get_all_etfs(&self) -> Result<impl Iterator<Item = Result<EtfData, SqliteError>> + use<'_>, SqliteError> {
        let query = "SELECT id, isin, name, proportion, cumulative FROM etf";
    
        let statement = self.connection.prepare(query)?;
    
        Ok(statement.into_iter().map(|row| row.map(|row| {
            let id: &str = row.read("id");
            let isin: &str = row.read("isin");
            let name: &str = row.read("name");
            let proportion: f64 = row.read("proportion");
            let cumulative: i64 = row.read("cumulative");
            
            EtfData::new(id.to_string(), isin.to_string(), name.to_string(), proportion, cumulative)
        })))
    }

    pub fn get_etf(&self, etf_id: &str) -> Result<Option<EtfData>, SqliteError> {
        let query = "SELECT id, isin, name, proportion, cumulative FROM etf WHERE id = :id";
        let mut statement = self.connection.prepare(query)?;
        statement.bind::<&[(_, Value)]>(&[(":id", etf_id.into())])?;

        statement.into_iter().map(|row| row.map(|row| {
            let id: &str = row.read("id");
            let isin: &str = row.read("isin");
            let name: &str = row.read("name");
            let proportion: f64 = row.read("proportion");
            let cumulative: i64 = row.read("cumulative");
            
            EtfData::new(id.to_string(), isin.to_string(), name.to_string(), proportion, cumulative)
        })).next().transpose()
    }
    
    pub fn update_proportion(&self, etf_id: &str, proportion: f64) -> Result<(), SqliteError>{
        let query = "
            UPDATE etf
            SET proportion = :proportion
            WHERE id = :id;
        ";
        let mut statement = self.connection.prepare(query)?;
        statement.bind::<&[(_, Value)]>(&[(":proportion", proportion.into()), (":id", etf_id.into())])?;
        statement.next()?;
        Ok(())
    }

    pub fn update_cumulative(&self, etf_id: &str, amount: i64) -> Result<(), SqliteError>{
        let query = "
            UPDATE etf
            SET cumulative = :amount
            WHERE id = :id;
        ";
        let mut statement = self.connection.prepare(query)?;
        statement.bind::<&[(_, Value)]>(&[(":amount", amount.into()), (":id", etf_id.into())])?;
        statement.next()?;
        Ok(())    
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect() {
        let db = Database::new(":memory:").unwrap();
        
        db.add_etf(EtfData::new("AGGG.L".into(), "ISIN".into(), "NAME ETF".into(), 0.9, 100)).unwrap();

        db.update_proportion("AGGG.L", 0.7).unwrap();
        db.update_cumulative("AGGG.L", 123).unwrap();
        
        let p = db.get_all_etfs().unwrap();
        for etf in p {
            let etf = etf.unwrap();
            println!("{} {} {}", etf.name, etf.proportion, etf.cumulative);
        }

        let p = db.get_etf("AGGG.L").unwrap().unwrap();
        println!("{} {} {}", p.name, p.proportion, p.cumulative);

        let j = db.get_etf("random id").unwrap();
        assert!(j.is_none());
    }
}
