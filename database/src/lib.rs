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
            CREATE TABLE IF NOT EXISTS budget (id INTEGER PRIMARY KEY, budget INTEGER);
        ";
        connection.execute(query)?;

        {
            let query = "
                INSERT OR IGNORE INTO budget (id, budget)
                values (0, 0);
            ";
            let mut statement = connection.prepare(query)?;
            statement.next()?;
        }

        let db = Database { connection };
        Ok(db)
    }

    pub fn add_etf(&self, etf: EtfData) -> Result<(), SqliteError> {
        let query = "
            INSERT OR REPLACE INTO etf (id, isin, name, proportion, cumulative) 
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

    pub fn remove_etf(&self, etf_id: String) -> Result<(), SqliteError> {
        let query = "
            DELETE FROM etf WHERE id = :id;
        ";
        let mut statement = self.connection.prepare(query)?;
        statement.bind::<&[(_, Value)]>(&[
            (":id", etf_id.into()),
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

    pub fn set_budget(&self, budget: i64) -> Result<(), SqliteError> {
        let query = "
            INSERT OR REPLACE INTO budget (id, budget)
            values (0, :budget);
        ";
        let mut statement = self.connection.prepare(query)?;
        statement.bind::<&[(_, Value)]>(&[(":budget", budget.into())])?;
        statement.next()?;
        Ok(())
    }

    pub fn get_budget(&self) -> Result<Option<i64>, SqliteError> {
        let query = "
            SELECT budget from budget WHERE id = 0;
        ";
        let statement = self.connection.prepare(query)?;
        statement.into_iter().map(|row| row.map(|row| {
            let budget: i64 = row.read("budget");
            budget
        })).next().transpose()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect() {
        let db = Database::new("db").unwrap();
        
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

        db.add_etf(EtfData::new("AGGG.L".into(), "ISIN".into(), "NAME ETF".into(), 0.1, 10)).unwrap();
        let p = db.get_all_etfs().unwrap();
        for etf in p {
            let etf = etf.unwrap();
            println!("{} {} {}", etf.name, etf.proportion, etf.cumulative);
        }

        db.set_budget(500).unwrap();
        let b = db.get_budget().unwrap().unwrap();
        println!("budget: {b}");
        db.set_budget(50).unwrap();
        let b = db.get_budget().unwrap().unwrap();
        println!("budget: {b}");
    }

    #[test]
    fn test_set_budget() {
        let db = Database::new("db").unwrap();
        db.set_budget(42).unwrap();
    }

    #[test]
    fn test_get_budget() {
        let db = Database::new("db").unwrap();
        let b = db.get_budget().unwrap().unwrap();
        println!("{b}")
    }
}
