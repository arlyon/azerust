AUTH=auth
CHARACTERS=characters

mkdir -p db
wget https://raw.githubusercontent.com/TrinityCore/TrinityCore/3.3.5/sql/create/create_mysql.sql -O db/00_databases.sql
wget https://raw.githubusercontent.com/TrinityCore/TrinityCore/3.3.5/sql/base/${AUTH}_database.sql -O db/01_$AUTH.sql
wget https://raw.githubusercontent.com/TrinityCore/TrinityCore/3.3.5/sql/base/${CHARACTERS}_database.sql -O db/02_$CHARACTERS.sql

sed -i '' "1s/^/use $AUTH;\\n/g" db/01_$AUTH.sql
sed -i '' "1s/^/use $CHARACTERS;\\n/g" db/02_$CHARACTERS.sql
