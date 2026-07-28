update request_logs
set error_message = $2
where id = $1
