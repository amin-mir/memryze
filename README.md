## MEMRYZE
Are you learning a new language? Want an efficient way for memorizing the countless new words, phrases
and sentences you encounter every day? Give this a try!

### TODO
* Use `env!` macro to read the server URL (localhost if value is empty). `env!` evaluates the 
env var in compile time and replaces it with a `&'static str`.
* Distinguish users in the database
* Implement token-based login in server
* Change the client to ask for login token and store in secure local storage

### Useful Queries for testing

```sql
-- reset last_shown_at for all qa.
UPDATE qa
SET last_shown_at = NULL;
```
