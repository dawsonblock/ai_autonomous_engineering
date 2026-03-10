# Basic two-node diagram
node "API Gateway" as api
node "Database" as db
api -> db : "SQL queries"
