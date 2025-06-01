CREATE TABLE IF NOT EXISTS quotes (
    id VARCHAR(255) PRIMARY KEY NOT NULL,
    whos_there VARCHAR(255) NOT NULL,    
    answer_who TEXT NOT NULL,           
    source VARCHAR(512) NOT NULL         
);

CREATE TABLE IF NOT EXISTS quote_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, 
    quote_id VARCHAR(255) NOT NULL,
    tag VARCHAR(255) NOT NULL,
    FOREIGN KEY (quote_id) REFERENCES quotes(id) ON DELETE CASCADE, 
    UNIQUE (quote_id, tag) 
);