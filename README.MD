# Telegram Bot for check WB Warehouse Coefficients
###### A bot for checking supply coefficients for Wildberries warehouses using the api on Rust language

 1. Clone the repository  
 ```git clone https://github.com/Polchasa/WbWarehouseCoefficients.git``` 
 2. Create a file.env in the root of the project and add your bot's token to it, which can be obtained from [BotFather](https://telegram.me/BotFather  
 ```TELOXIDE_TOKEN=1234567890:ABCDefGhkLm6N-V9BuoURUB3edZltnG07Zg```
 3. In file `commands_handlers.rs` set admin login at line 29  
 `let  admin_username = "SET_YOUR_LOGIN_HERE".to_string();`
 4. Start bot with `cargo run`
