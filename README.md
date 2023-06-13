# **TP2**

## **Integrantes**

- Dituro, Celeste
- Lopez, Victoria
- Czop, Santiago

## **Análisis y Resolución del Problema**

### Conexiones entre locales

El programa consta de 4 aplicaciones:

- Cafeteras: conformada por 1 actor por cada cafetera del local.
- Servidor del local: conformada por 3 threads. Uno de ellos ejecuta la elección del lider, otro escucha los mensajes que envian las cafeteras al servidor local y el último escucha los mensajes que le envian los servidores de las otras sucursales del local al servidor del local.
- Programa para desconectar un servidor
- Programa para conectar un servidor

![tp2-concu-Conexión entre locales drawio](https://github.com/concurrentes-fiuba/2023-1c-tp2-concu-csv/assets/67125933/5da54256-d809-4e2c-9550-ccf699ca8411)

El envio de mensajes entre los servidores locales se realiza mediante el algoritmo ring, es decir, los servidores van a estar conectados entre sí formando un anillo en donde un servidor va a mantener sólo 2 conexiones: 1 con el servidor que le antecede y 1 con el servidor que le precede.

### Mensajes entre las cafeteras y un servidor local

Las cafeteras se comunican con el servidor local por medio de sockets. Hay 4 posibles mensajes que las cafeteras les pueden enviar al servidor:

- **BLOCK** *id_cliente* *id_shop*: para bloquear la cuenta del cliente asociado. La cuenta de un cliente se bloquea sólo si desea pagar con puntos.
- **COMPLETE** *id_cliente* *puntos* *forma_de_pago* *id_shop*: se envia si la cafetera pudo procesar correctamente el pedido y tiene como objetivos actualizar los puntos de la cuenta del cliente y desbloquearla.
- **FAILURE** *id_cliente* *id_shop*: se envia si la cafetera no pudo procesar correctamente el pedido y el objetivo es desbloquear la cuenta del cliente asociado.

Por otro lado, los mensajes que puede recibir una cafetera de un servidor local:

- **ACK**: el servidor recibió el mensaje.
- **CLIENT ALREADY BLOCKED** *id_cliente*: la cuenta del cliente que se quiere usar ya está siendo usada por lo que no se puede usar.
- **NOT ENOUGH POINTS** *id_cliente*: se recibe si el cliente quiere pagar con puntos pero no tiene los puntos necesarios para pagar el pedido.

### Caso: El cliente puede pagar con puntos o dinero y el pedido es procesado correctamente por la cafetera

![tp2-concu-Sec  1 drawio](https://github.com/concurrentes-fiuba/2023-1c-tp2-concu-csv/assets/67125933/f7ef1f9d-2c7c-432e-8df3-36c66f5a29c9)

Cuando una cafetera toma un pedido en el que el cliente quiere pagar con puntos, realiza el siguiente intercambio de mensajes con el servidor:

1. Envia mensaje **BLOCK** al servidor

2. Espera mensaje **ACK** del servidor.
  
3. Procesa pedido.

4. Envía mensaje **COMPLETE** al servidor

5. Recibe mensaje **ACK** del servidor.

### Caso: El cliente quiere pagar con puntos pero no tiene suficientes puntos y el pedido es procesado correctamente por la cafetera

![tp2-concu-Sec  2 drawio](https://github.com/concurrentes-fiuba/2023-1c-tp2-concu-csv/assets/67125933/0e14652e-fbbf-4732-a953-577744993c0e)

Cuando una cafetera toma un pedido en el que el cliente quiere pagar con puntos, realiza el siguiente intercambio de mensajes con el servidor:

1. Envia mensaje **BLOCK** al servidor.

2. Espera mensaje **ACK** del servidor.
  
3. Procesa pedido.

4. Envía mensaje **COMPLETE** al servidor.

5. Recibe mensaje **NOT ENOUGH POINTS** del servidor: el cliente no tiene los puntos necesarios para pagar el pedido con puntos por lo que va a tener que pagarlo con dinero.

6. Envía mensaje **COMPLETE** al servidor cambiando el método de pago del pedido de puntos a dinero.

7. Recibe mensaje **ACK** del servidor.

### Caso: El cliente puede pagar con puntos o dinero pero se pierde el ACK del bloqueo de la cuenta del lider al servidor local

![tp2-concu-Sec  3 drawio](https://github.com/concurrentes-fiuba/2023-1c-tp2-concu-csv/assets/67125933/e11c6e1c-0c54-4d3b-befb-13c5c1b6282c)

Siempre que una cafetera le envie un mensaje al servidor del local y no le llega el ACK del mismo, la cafetera va a intentar enviar el mensaje una cantidad configurable de veces.

Usamos conexiones UDP para disminuir la cantidad de conexiones a establecer entre las entidades del sistema pero implementamos el servicio de confirmación de mensajes para chequear que no haya pérdidas de mensajes.

## **Hipótesis**

- Los servidores locales no se caen permanentemente.
- La conexión entre las cafeteras y el servidor del local siempre se puede establecer.

## **Ejecución del Programa**

Para ejecutar cada servidor local es necesario correr:

```cargo run --bin local_server <id de la sucursal> <cantidad total de sucursales>```

Para ejecutar las cafeteras es necesario correr:
```cargo run --bin coffee_machine <archivo con ordenes> <id de la sucursal>```

## **Casos de Prueba**
