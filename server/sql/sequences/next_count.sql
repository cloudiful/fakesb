update rules
set sequence_count = sequence_count + 1
where id = $1
returning sequence_count
