# log-ql

> Log file query language


## About this project

The purpose of this project is to provide a language which enables a user to query for information on log files -- similar to SQL which provides a uniform way of accessing relational data.

## Examples

> Provide me with all messages and all their fields from a log file where the severity is warning:

```
SELECT * FROM 'app.log' WHERE severity = 'warning'
```

> Provide me with just the date and the message from the last 10 log messages:

```
SELECT date, message FROM 'app.log' LIMIT LAST 10;
```

> Provide me with just the date and the message from the first 10 log messages:

```
SELECT date, message FROM 'app.log' LIMIT 10;
```