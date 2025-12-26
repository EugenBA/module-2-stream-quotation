# module-2 стримминг котировок

## Серверная часть
Многопоточный сервер для обработки запросов на получение котировок акций TCP/UDP (udp для стримминга).
Запуск сервера:
quote-server <server_addr> <udp-port> <ticker-file> <log-level>

<server_addr> - адрес и порт на прослушивание данных 
<udp-port> - udp порт для обмена по протоколу UDP
<ticker-file>  - файл котировок (с разделителем "\n") для генерации котировок
<log-level> - уровень логирования (info, debug, warn, error)

Пример запуска сервера:
quote-server localhost:8210 55505 tickers.txt info

Логирование отравляется в stdout

## Клиентская часть
Клиент для запроса котировок акций.
Запуск клиента:
Многопоточный клиент, поддерживает отправление данных PING для контроля работы со стороны сервера
quote-client <server_addr <udp-port> <ticker-file> <log-level>

Пример запуска сервера:
quote-server localhost:8210 55500 tickers.txt info
<server_addr> - адрес и порт сервера котировок
<udp-port> - udp порт для обмена по протоколу UDP
<ticker-file>  - файл запроса котировок (с разделителем "\n")
<log-level> - уровень логирования (info, debug, warn, error)

Логирование отравляется в файл: quote-client.log
