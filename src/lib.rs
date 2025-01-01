use std::ffi::{c_char, CStr, CString};
use std::fmt::Display;
use std::mem;
use std::sync::{LazyLock, Mutex};
use database::{Database, EtfData, SqliteError};
use derive_new::new;
use investment_planner::{EtfSetting, Investment, Settings};
use tokio::runtime::Runtime;
use yahoo_finance_info::YahooError;
use futures::future;

static RT: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().expect("Could not initialize tokio async runtime"));
static DB: LazyLock<Mutex<Database>> = LazyLock::new(|| Mutex::new(Database::new("db").expect("Could not create database from 'db' file")));

fn c_char_ptr_to_string(c_ptr: *const c_char) -> String {
    unsafe {
        CStr::from_ptr(c_ptr)
    }.to_str().expect("could not parse C string as UTF-8").to_string()
}

fn string_to_c_char_ptr(xs: String) -> *const c_char {
    CString::new(xs).expect("unexpectedly found 0 byte in String").into_raw()
}

#[repr(C)]
#[derive(new)]
pub struct CInvestment {
    pub etf_id: *const c_char,
    pub name: *const c_char,
    pub quantity: i64,
    pub price: i64
}
impl From<Investment> for CInvestment {
    fn from(investment: Investment) -> Self {
        CInvestment::new(string_to_c_char_ptr(investment.etf_id), string_to_c_char_ptr(investment.name), investment.quantity, investment.price)
    }
}

#[repr(C)]
#[derive(new)]
pub struct CInvestments {
    pub investments: *const CInvestment,
    pub length: usize,
}
impl From<Vec<Investment>> for CInvestments {
    fn from(investments: Vec<Investment>) -> Self {
        let mut c_investments = investments.into_iter().map(|i| CInvestment::from(i)).collect::<Vec<_>>();
        c_investments.shrink_to_fit();
        let len = c_investments.len();
        let c_investments_ptr = c_investments.as_ptr();
        mem::forget(c_investments);

        CInvestments::new(c_investments_ptr, len)
    }
}

#[repr(C)]
#[derive(Clone, Copy, new)]
pub struct CEtfSetting {
    pub id: *const c_char,
    pub isin: *const c_char,
    pub name: *const c_char,
    pub ideal_proportion: f64,
    pub cumulative: i64
}
impl CEtfSetting {
    fn etf_setting(&self) -> EtfSetting {
        EtfSetting::new(c_char_ptr_to_string(self.id), c_char_ptr_to_string(self.isin), c_char_ptr_to_string(self.name), self.ideal_proportion, self.cumulative)
    }
}
impl From<EtfSetting> for CEtfSetting {
    fn from(etf_setting: EtfSetting) -> Self {
        CEtfSetting::new(string_to_c_char_ptr(etf_setting.id), string_to_c_char_ptr(etf_setting.isin), string_to_c_char_ptr(etf_setting.name), etf_setting.ideal_proportion, etf_setting.cumulative)
    }
}

#[repr(C)]
#[derive(new)]
pub struct CSettings {
    pub budget: i64,
    pub etf_settings: *const CEtfSetting,
    pub num_etf_settings: usize
}

impl CSettings {
    fn settings(&self) -> Settings {
        let budget = self.budget;

        let mut etf_settings = vec![];
        for i in 0..self.num_etf_settings {
            let etf_setting = unsafe {
                *self.etf_settings.add(i)
            }.etf_setting();
            etf_settings.push(etf_setting);
        }
        
        Settings::new(budget, etf_settings)
    }
}
impl From<Settings> for CSettings {
    fn from(settings: Settings) -> Self {
        let mut c_etf_settings = settings.etf_settings
            .into_iter()
            .map(|etf| 
                CEtfSetting::from(etf)
            )
            .collect::<Vec<_>>();
        c_etf_settings.shrink_to_fit();
        let len = c_etf_settings.len();
        let etf_settings_ptr = c_etf_settings.as_ptr();
        mem::forget(c_etf_settings);

        CSettings::new(settings.budget, etf_settings_ptr, len)
    }
}

#[repr(C)]
#[derive(new)]
pub struct CEtfInfo {
    pub id: *const c_char,
    pub name: *const c_char,
    pub isin: *const c_char,
}

fn check_result<T, E: Display, Ret, RetErr: Fn() -> Ret, RetOk: Fn(T) -> Ret>(result: Result<T, E>, ret_err: RetErr, ret_ok: RetOk) -> Ret {
    match result {
        Err(e) => {
            eprintln!("{e}");
            ret_err()
        }
        Ok(t) => ret_ok(t)
    }
}

fn check_option<T, Ret, RetNone: Fn() -> Ret, RetSome: Fn(T) -> Ret>(option: Option<T>, ret_none: RetNone, ret_some: RetSome) -> Ret {
    match option {
        None => ret_none(),
        Some(t) => ret_some(t)
    }
}

#[no_mangle]
pub extern "C" fn search_etf_info(etf_isin_ptr: *const c_char) -> *const CEtfInfo {
    let etf_isin = c_char_ptr_to_string(etf_isin_ptr);

    let result = RT.block_on(yahoo_finance_info::search_etf_isin(&etf_isin));
    check_result(result, 
        || std::ptr::null(), 
        |xs| {
            if xs.len() > 1 {
                eprintln!("found more than 1 result while searching for {}. Returning null.", etf_isin);
                return std::ptr::null();
            }
            let opt = xs.into_iter().next();
            check_option(opt, 
                || {
                    eprintln!("could not find an etf with isin = {}", etf_isin);
                    std::ptr::null()
                }, 
                |x| {
                    let etf_info = CEtfInfo::new(string_to_c_char_ptr(x.ticker), string_to_c_char_ptr(x.name), string_to_c_char_ptr(x.isin));
                    Box::into_raw(Box::new(etf_info))
                })
        })
}

#[no_mangle]
pub extern "C" fn get_price_of(etf_id_ptr: *const c_char) -> f64 {
    let etf_id = unsafe {
        CStr::from_ptr(etf_id_ptr)
    }.to_str().expect("could not parse C string as utf-8");
 
    let result = RT.block_on(yahoo_finance_info::get_price_of(&etf_id.to_string()));
    check_result(result, || f64::NAN, |price| price)
}

fn get_settings_from_db() -> Result<Settings, SqliteError> {
    let db: std::sync::MutexGuard<'_, Database> = DB.lock().unwrap();

    let budget = db.get_budget()?.expect("database should always have a budget");
    let etf_settings = db
        .get_all_etfs()?
        .map(|etf| 
            etf.map(|etf| EtfSetting::new(etf.id, etf.isin, etf.name, etf.proportion, etf.cumulative))
        ).collect::<Result<Vec<_>, _>>()?;
    Ok(Settings::new(budget, etf_settings))
}

async fn get_prices(settings: &Settings) -> Result<Vec<f64>, YahooError> {
    let futures = settings.etf_settings.iter().map(|etf| yahoo_finance_info::get_price_of(&etf.id));
    future::try_join_all(futures).await
}

#[no_mangle]
pub extern "C" fn suggest_investments() -> CInvestments {    
    let settings = match get_settings_from_db() {
        Err(e) => {
            eprintln!("{e}");
            return CInvestments::new(std::ptr::null(), 0);
        }
        Ok(settings) => settings
    };

    let prices = RT.block_on(get_prices(&settings));
    let prices = match prices {
        Err(e) => {
            eprintln!("{e}");
            return CInvestments::new(std::ptr::null(), 0);
        }
        Ok(prices) => prices.into_iter().map(|p: f64| p * 100.0 /* convert euros to cents */).collect::<Vec<_>>() 
    };
    
    let xs = investment_planner::next_investments(settings, &prices);
    CInvestments::from(xs)
}

#[no_mangle]
pub extern "C" fn persist_settings(settings: *const CSettings) -> i64 {
    let settings = unsafe {&*settings};
    let settings = settings.settings();
    
    let db = DB.lock().unwrap();
    if let Err(e) = db.set_budget(settings.budget) {
        eprintln!("{e}");
        return  -1;
    }
    match db.get_all_etfs() {
        Err(e) => {
            eprintln!("{e}");
            return  -2;
        }
        Ok(etfs) => {
            for etf in etfs {
                match etf {
                    Err(e) => {
                        eprintln!("{e}");
                        return  -3;
                    }
                    Ok(etf) => {
                        if let Err(e) = db.remove_etf(etf.id) {
                            eprintln!("{e}");
                            return  -4;
                        }
                    }
                }
            }
        }
    }
    for etf in settings.etf_settings {
        if let Err(e) = db.add_etf(EtfData::new(etf.id, etf.isin, etf.name, etf.ideal_proportion, etf.cumulative)) {
            eprintln!("{e}");
            return -1;
        }
    }

    return 0;
}

#[no_mangle]
pub extern "C" fn get_settings() -> *const CSettings {
    let settings = match get_settings_from_db() {
        Err(e) => {
            eprintln!("{e}");
            return std::ptr::null();
        }
        Ok(settings) => settings
    };

    Box::into_raw(Box::new(CSettings::from(settings)))
}