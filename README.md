# module-2 стримминг котировок


## Серверная часть
Многопоточный сервер для обработки запросов на получение котировок акций TCP/UDP (udp для стримминга).

### Сборка:

cargo build --package quote-app --bin quote-server --features server

### Запуск сервера:
quote-server <server_addr> <udp-port> <ticker-file> <log-level> <log-file>

- &lt;server_addr&gt; - адрес и порт на прослушивание данных 
- &lt;udp-port&gt; - udp порт для обмена по протоколу UDP
- &lt;ticker-file&gt;  - файл котировок (с разделителем "\n") для генерации котировок
- &lt;log-level&gt; - уровень логирования (info, debug, warn, error)
- &lt;log-file&gt; - файл для логирования (по умолчанию quote-server.log)

Логирование по умолчанию отравляется в файл: quote-server.log

Пример запуска сервера:
quote-server -s 127.0.0.1:8210 -u 55505 -t tickers_request.txt -l info

## Клиентская часть
Клиент для запроса котировок акций.
Многопоточный клиент, поддерживает отправление данных PING для контроля работы со стороны сервера

### Сборка:
cargo build --package quote-app --bin quote-client --features client

### Запуск клиента:
quote-client <server_addr <udp-port> <ticker-file> <log-level> <log-file>

Пример запуска клиента:
quote-client -s 127.0.0.1:8210 -u 55500 -t tickers.txt -l info
- &lt;server_addr&gt; - адрес и порт сервера котировок
- &lt;udp-port&gt; - udp порт для обмена по протоколу UDP
- &lt;ticker-file&gt;  - файл запроса котировок (с разделителем "\n")
- &lt;log-level&gt; - уровень логирования (info, debug, warn, error)
- &lt;log-file&gt; - файл для логирования (по умолчанию quote-client.log)

Логирование по умолчанию отравляется в файл: quote-client.log

## Пример вывода данных:

![img.png](img.png)

## Файлы tickers:
- tickers.txt - файл котировок для сервера
- tickers_request.txt - файл котировок для запроса клиентом
- tickers_request_1.txt - файл котировок для запроса клиентом

