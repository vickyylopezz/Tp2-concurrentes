# **TP2**
## **Integrantes**
- Dituro, Celeste
- Lopez, Victoria
- Czop, Santiago

## **Análisis del Problema**
### Conexiones entre locales
![tp2-concu-Conexión entre locales drawio](https://github.com/concurrentes-fiuba/2023-1c-tp2-concu-csv/assets/67125933/5da54256-d809-4e2c-9550-ccf699ca8411)

### Mensajes entre las cafeteras y un servidor local
![tp2-concu-Mensajes Cafetera-ServidorLocal drawio](https://github.com/concurrentes-fiuba/2023-1c-tp2-concu-csv/assets/67125933/5577e2d3-1eec-4e20-83e6-0a6bff4e3031)

### Caso: El cliente puede pagar con puntos o dinero
![tp2-concu-Sec  1 drawio](https://github.com/concurrentes-fiuba/2023-1c-tp2-concu-csv/assets/67125933/f7ef1f9d-2c7c-432e-8df3-36c66f5a29c9)

### Caso: El cliente quiere pagar con puntos pero no tiene suficientes puntos
![tp2-concu-Sec  2 drawio](https://github.com/concurrentes-fiuba/2023-1c-tp2-concu-csv/assets/67125933/0e14652e-fbbf-4732-a953-577744993c0e)

### Caso: El cliente puede pagar con puntos o dinero pero se pierde el ACK del bloqueo de la cuenta del lider al servidor local
![tp2-concu-Sec  3 drawio](https://github.com/concurrentes-fiuba/2023-1c-tp2-concu-csv/assets/67125933/e11c6e1c-0c54-4d3b-befb-13c5c1b6282c)

## **Hipótesis**
- Los servidores locales no se caen permanentemente.

## **Ejecución del Programa**

Para ejecutar cada servidor local es necesario correr:
```
cargo run --bin local_server <id de la sucursal> <cantidad total de sucursales>
```

## **Resolución del Problema**

## **Casos de Prueba**

