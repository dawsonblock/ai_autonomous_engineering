# Test all four node types
node "Web App" as web [type: service]
node "MySQL" as mysql [type: database]
node "SQS Queue" as sqs [type: queue]
node "Twilio SMS" as twilio [type: external]

web -> mysql : "read/write"
web -> sqs : "enqueue jobs"
sqs -> twilio : "send SMS"
