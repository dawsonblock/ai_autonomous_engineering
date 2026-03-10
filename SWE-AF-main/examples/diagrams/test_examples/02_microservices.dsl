# Microservices architecture
node "API Gateway" as gateway
node "Auth Service" as auth [type: service]
node "User Service" as users [type: service]
node "Order Service" as orders [type: service]
node "PostgreSQL" as pg [type: database]
node "Redis Cache" as redis [type: database]
node "RabbitMQ" as mq [type: queue]
node "Stripe" as stripe [type: external]

gateway -> auth : "authenticate"
gateway -> users : "user ops"
gateway -> orders : "order ops"
auth -> redis : "session tokens"
users -> pg : "CRUD"
orders -> pg : "CRUD"
orders -> mq : "order.placed"
orders -> stripe : "charge card"
mq -> users : "notify user"
