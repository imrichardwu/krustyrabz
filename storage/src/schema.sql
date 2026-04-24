CREATE TABLE UserAccount( 
	id INTEGER PRIMARY KEY, 
	username TEXT NOT NULL UNIQUE, 
	token_balance REAL, 
	rounds_played INTEGER, 
	pots_won INTEGER, 
	number_folds INTEGER
); 

CREATE INDEX rounds_played_statistics_idx INDEX(rounds_played); 
CREATE INDEX pots_won_statistics_idx INDEX(pots_won); 
CREATE INDEX number_folds_statistics_idx INDEX(number_folds); 

