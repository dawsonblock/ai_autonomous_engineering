# Data pipeline architecture
node "Kafka" as kafka [type: queue]
node "Spark Streaming" as spark [type: service]
node "Data Lake (S3)" as s3 [type: database]
node "Snowflake" as snowflake [type: database]
node "dbt Transform" as dbt [type: service]
node "Metabase Dashboard" as dashboard [type: service]
node "Airflow" as airflow [type: service]

kafka -> spark : "consume events"
spark -> s3 : "write parquet"
airflow -> dbt : "trigger daily"
dbt -> snowflake : "transform"
s3 -> snowflake : "load raw"
snowflake -> dashboard : "query"
