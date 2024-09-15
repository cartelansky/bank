#![allow(dead_code)]

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Clone)]
struct Currency {
    code: String,
    name: String,
    symbol: String,
}

#[derive(Clone)]
struct Transaction {
    date: DateTime<Utc>,
    transaction_type: String,
    amount: f64,
    balance: f64,
    currency: Currency,
}

struct Account {
    id: u32,
    owner: String,
    balances: HashMap<String, f64>,
    pin: String,
    transactions: Vec<Transaction>,
}

struct Bank {
    accounts: HashMap<u32, Account>,
    next_id: u32,
    currencies: HashMap<String, Currency>,
}

impl Bank {
    fn verify_pin(&self, account_id: u32, pin: &str) -> Result<(), String> {
        let account = self
            .accounts
            .get(&account_id)
            .ok_or_else(|| format!("Hesap bulunamadı: {}", account_id))?;

        if account.pin == pin {
            Ok(())
        } else {
            Err("Geçersiz PIN".to_string())
        }
    }

    fn new() -> Self {
        let mut currencies = HashMap::new();
        currencies.insert(
            "TRY".to_string(),
            Currency {
                code: "TRY".to_string(),
                name: "Türk Lirası".to_string(),
                symbol: "₺".to_string(),
            },
        );
        currencies.insert(
            "USD".to_string(),
            Currency {
                code: "USD".to_string(),
                name: "Amerikan Doları".to_string(),
                symbol: "$".to_string(),
            },
        );
        currencies.insert(
            "EUR".to_string(),
            Currency {
                code: "EUR".to_string(),
                name: "Euro".to_string(),
                symbol: "€".to_string(),
            },
        );

        Bank {
            accounts: HashMap::new(),
            next_id: 1,
            currencies,
        }
    }

    fn create_account(
        &mut self,
        owner: String,
        initial_balance: f64,
        currency_code: &str,
        pin: String,
    ) -> Result<u32, String> {
        if !self.currencies.contains_key(currency_code) {
            return Err(format!("Geçersiz para birimi: {}", currency_code));
        }

        let id = self.next_id;
        self.next_id += 1;
        let currency = self.currencies.get(currency_code).unwrap().clone();
        let mut balances = HashMap::new();
        balances.insert(currency_code.to_string(), initial_balance);

        let initial_transaction = Transaction {
            date: Utc::now(),
            transaction_type: "Hesap Açılışı".to_string(),
            amount: initial_balance,
            balance: initial_balance,
            currency: currency.clone(),
        };

        let account = Account {
            id,
            owner,
            balances,
            pin,
            transactions: vec![initial_transaction],
        };
        self.accounts.insert(id, account);
        Ok(id)
    }

    fn get_balance(&self, id: u32, currency_code: &str, pin: &str) -> Result<f64, String> {
        self.verify_pin(id, pin)?;
        let account = self
            .accounts
            .get(&id)
            .ok_or_else(|| format!("Hesap bulunamadı: {}", id))?;
        account
            .balances
            .get(currency_code)
            .cloned()
            .ok_or_else(|| format!("Bu para birimi için bakiye bulunamadı: {}", currency_code))
    }

    fn deposit(
        &mut self,
        id: u32,
        amount: f64,
        currency_code: &str,
        pin: &str,
    ) -> Result<f64, String> {
        self.verify_pin(id, pin)?;
        if !self.currencies.contains_key(currency_code) {
            return Err(format!("Geçersiz para birimi: {}", currency_code));
        }

        if let Some(account) = self.accounts.get_mut(&id) {
            let balance = account
                .balances
                .entry(currency_code.to_string())
                .or_insert(0.0);
            *balance += amount;
            let currency = self.currencies.get(currency_code).unwrap().clone();
            account.transactions.push(Transaction {
                date: Utc::now(),
                transaction_type: "Para Yatırma".to_string(),
                amount,
                balance: *balance,
                currency,
            });
            Ok(*balance)
        } else {
            Err(format!("Hesap bulunamadı: {}", id))
        }
    }

    fn withdraw(
        &mut self,
        id: u32,
        amount: f64,
        currency_code: &str,
        pin: &str,
    ) -> Result<f64, String> {
        self.verify_pin(id, pin)?;
        if !self.currencies.contains_key(currency_code) {
            return Err(format!("Geçersiz para birimi: {}", currency_code));
        }

        if let Some(account) = self.accounts.get_mut(&id) {
            let balance = account.balances.get_mut(currency_code).ok_or_else(|| {
                format!("Bu para birimi için bakiye bulunamadı: {}", currency_code)
            })?;
            if *balance >= amount {
                *balance -= amount;
                let currency = self.currencies.get(currency_code).unwrap().clone();
                account.transactions.push(Transaction {
                    date: Utc::now(),
                    transaction_type: "Para Çekme".to_string(),
                    amount: -amount,
                    balance: *balance,
                    currency,
                });
                Ok(*balance)
            } else {
                Err("Yetersiz bakiye".to_string())
            }
        } else {
            Err(format!("Hesap bulunamadı: {}", id))
        }
    }

    fn transfer(
        &mut self,
        from_id: u32,
        to_id: u32,
        amount: f64,
        currency_code: &str,
        pin: &str,
    ) -> Result<(), String> {
        self.verify_pin(from_id, pin)?;

        if from_id == to_id {
            return Err("Aynı hesaba transfer yapamazsınız".to_string());
        }

        if !self.accounts.contains_key(&from_id) {
            return Err(format!("Gönderen hesap bulunamadı: {}", from_id));
        }

        if !self.accounts.contains_key(&to_id) {
            return Err(format!("Alıcı hesap bulunamadı: {}", to_id));
        }

        if !self.currencies.contains_key(currency_code) {
            return Err(format!("Geçersiz para birimi: {}", currency_code));
        }

        let from_balance = self.get_balance(from_id, currency_code, pin)?;
        if from_balance < amount {
            return Err("Yetersiz bakiye".to_string());
        }

        // İşlemleri gerçekleştir
        let date = Utc::now();
        let currency = self.currencies.get(currency_code).unwrap().clone();

        if let Some(from_account) = self.accounts.get_mut(&from_id) {
            let balance = from_account.balances.get_mut(currency_code).unwrap();
            *balance -= amount;
            from_account.transactions.push(Transaction {
                date,
                transaction_type: format!("Transfer (Hesap {})", to_id),
                amount: -amount,
                balance: *balance,
                currency: currency.clone(),
            });
        }
        if let Some(to_account) = self.accounts.get_mut(&to_id) {
            let balance = to_account
                .balances
                .entry(currency_code.to_string())
                .or_insert(0.0);
            *balance += amount;
            to_account.transactions.push(Transaction {
                date,
                transaction_type: format!("Transfer (Hesap {})", from_id),
                amount,
                balance: *balance,
                currency: currency.clone(),
            });
        }

        Ok(())
    }
}

fn main() {
    let mut bank = Bank::new();

    loop {
        print_menu();
        let choice = get_input("Seçiminiz: ");

        match choice.as_str() {
            "1" => create_account(&mut bank),
            "2" => check_balance(&bank),
            "3" => deposit(&mut bank),
            "4" => withdraw(&mut bank),
            "5" => transfer(&mut bank),
            "6" => view_transaction_history(&bank),
            "7" => break,
            _ => println!("Geçersiz seçim. Lütfen tekrar deneyin."),
        }

        println!("\nDevam etmek için Enter'a basın...");
        io::stdin().read_line(&mut String::new()).unwrap();
    }

    println!("Banka uygulamasından çıkılıyor. İyi günler!");
}

fn create_account(bank: &mut Bank) {
    let owner = get_input("Hesap sahibinin adı: ");
    let initial_balance: f64 = get_input("Başlangıç bakiyesi: ").parse().unwrap_or(0.0);
    let currency_code = get_input("Para birimi (TRY/USD/EUR): ");
    let pin = get_input("4 haneli PIN belirleyin: ");
    if pin.len() != 4 || pin.parse::<u32>().is_err() {
        println!("Geçersiz PIN. 4 haneli bir sayı olmalıdır.");
        return;
    }
    match bank.create_account(owner.clone(), initial_balance, &currency_code, pin) {
        Ok(account_id) => println!("{} adına {} numaralı hesap oluşturuldu.", owner, account_id),
        Err(e) => println!("Hesap oluşturma hatası: {}", e),
    }
}

fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn print_menu() {
    println!("\n--- Banka Uygulaması ---");
    println!("1. Hesap Oluştur");
    println!("2. Bakiye Sorgula");
    println!("3. Para Yatır");
    println!("4. Para Çek");
    println!("5. Transfer Yap");
    println!("6. İşlem Geçmişini Görüntüle");
    println!("7. Çıkış");
}

fn view_transaction_history(bank: &Bank) {
    let account_id: u32 = get_input("Hesap numarası: ").parse().unwrap_or(0);
    let pin = get_input("PIN: ");

    match bank.accounts.get(&account_id) {
        Some(account) => {
            if account.pin == pin {
                for transaction in &account.transactions {
                    println!(
                        "{} - {}: {:.2} {} (Bakiye: {:.2} {})",
                        transaction.date,
                        transaction.transaction_type,
                        transaction.amount,
                        transaction.currency.symbol,
                        transaction.balance,
                        transaction.currency.symbol
                    );
                }
            } else {
                println!("Geçersiz PIN.");
            }
        }
        None => println!("Hesap bulunamadı."),
    }
}

fn check_balance(bank: &Bank) {
    let account_id: u32 = get_input("Hesap numarası: ").parse().unwrap_or(0);
    let currency_code = get_input("Para birimi (TRY/USD/EUR): ");
    let pin = get_input("PIN: ");
    match bank.get_balance(account_id, &currency_code, &pin) {
        Ok(balance) => println!("Hesap bakiyesi: {:.2} {}", balance, currency_code),
        Err(e) => println!("Hata: {}", e),
    }
}

fn deposit(bank: &mut Bank) {
    let account_id: u32 = get_input("Hesap numarası: ").parse().unwrap_or(0);
    let amount: f64 = get_input("Yatırılacak miktar: ").parse().unwrap_or(0.0);
    let currency_code = get_input("Para birimi (TRY/USD/EUR): ");
    let pin = get_input("PIN: ");
    match bank.deposit(account_id, amount, &currency_code, &pin) {
        Ok(new_balance) => println!(
            "İşlem başarılı. Yeni bakiye: {:.2} {}",
            new_balance, currency_code
        ),
        Err(e) => println!("Hata: {}", e),
    }
}

fn withdraw(bank: &mut Bank) {
    let account_id: u32 = get_input("Hesap numarası: ").parse().unwrap_or(0);
    let amount: f64 = get_input("Çekilecek miktar: ").parse().unwrap_or(0.0);
    let currency_code = get_input("Para birimi (TRY/USD/EUR): ");
    let pin = get_input("PIN: ");
    match bank.withdraw(account_id, amount, &currency_code, &pin) {
        Ok(new_balance) => println!(
            "İşlem başarılı. Yeni bakiye: {:.2} {}",
            new_balance, currency_code
        ),
        Err(e) => println!("Hata: {}", e),
    }
}

fn transfer(bank: &mut Bank) {
    let from_id: u32 = get_input("Gönderen hesap numarası: ").parse().unwrap_or(0);
    let to_id: u32 = get_input("Alıcı hesap numarası: ").parse().unwrap_or(0);
    let amount: f64 = get_input("Transfer miktarı: ").parse().unwrap_or(0.0);
    let currency_code = get_input("Para birimi (TRY/USD/EUR): ");
    let pin = get_input("Gönderen hesap PIN: ");

    match bank.transfer(from_id, to_id, amount, &currency_code, &pin) {
        Ok(_) => {
            println!("Transfer başarılı.");
            if let Ok(from_balance) = bank.get_balance(from_id, &currency_code, &pin) {
                println!(
                    "Gönderen hesap yeni bakiye: {:.2} {}",
                    from_balance, currency_code
                );
            }
            if let Ok(to_balance) = bank.get_balance(to_id, &currency_code, &pin) {
                println!(
                    "Alıcı hesap yeni bakiye: {:.2} {}",
                    to_balance, currency_code
                );
            }
        }
        Err(e) => println!("Transfer hatası: {}", e),
    }
}
