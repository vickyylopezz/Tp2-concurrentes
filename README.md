# **TP2**

## **Integrantes**

- Dituro, Celeste
- Lopez, Victoria
- Czop, Santiago

## **Análisis y Resolución del Problema**

### Conexiones entre locales

![tp2-concu-Conexión entre locales drawio](https://github.com/concurrentes-fiuba/2023-1c-tp2-concu-csv/assets/67125933/5da54256-d809-4e2c-9550-ccf699ca8411)

Hay 1 thread por cada servidor de un local, 1 actor por cada cafetera de un local y 1 thread en donde se realiza la elección del lider. El envio de mensajes entre los servidores locales se realiza mediante el algoritmo ring, es decir, los servidores van a estar conectados entre sí formando un anillo en donde un servidor va a mantener sólo 2 conexiones: 1 con el servidor que le antecede y 1 con el servidor que le precede. Utilizamos el protocolo de transporte UDP con el servicio de pérdida de paquetes para establecer las conexiones entre los procesos con el objeto de disminuir la cantidad de conexiones totales a establecer. Tenemos una aplicación por el conjunto de cafeteras que le envian mensaje al servidor del local que le corresponde y una aplicación por cada servidor de un local.

### Mensajes entre las cafeteras y un servidor local

![tp2-concu-Mensajes Cafetera-ServidorLocal drawio](https://github.com/concurrentes-fiuba/2023-1c-tp2-concu-csv/assets/67125933/5577e2d3-1eec-4e20-83e6-0a6bff4e3031)

Las cafeteras se comunican con el servidor local por medio de sockets. Hay 3 posibles mensajes que las cafeteras les pueden enviar al servidor: 

- **BLOCK** *cliente_id*: se envia para bloquear la cuenta del cliente asociado.
- **COMPLETE** *cliente_id* *puntos* *forma_de_pago*: se envia si la cafetera pudo procesar correctamente el pedido y tiene como objetivos actualizar los puntos de la cuenta del cliente y desbloquearla.
- **FAILURE** *cliente_id*: se envia si la cafetera no pudo procesar correctamente el pedido y el objetivo es desbloquear la cuenta del cliente asociado.
  
### Caso: El cliente puede pagar con puntos o dinero

![tp2-concu-Sec  1 drawio](https://github.com/concurrentes-fiuba/2023-1c-tp2-concu-csv/assets/67125933/f7ef1f9d-2c7c-432e-8df3-36c66f5a29c9)

Cuando una cafetera toma un pedido, realiza el siguiente intercambio de mensajes con el servidor:

1. Envia mensaje **BLOCK** al servidor: para que se bloquee la cuenta del cliente asociado así otras personas no pueden utilizar los puntos de esta cuenta de manera simultánea.

2. Espera mensaje **ACK** del servidor: si el mensaje ACK le llega, continua con el procesamiento del pedido. Caso contrario, va a intentar enviar el mensaje un cantidad que se puede establecer de veces.

- Si el servidor responde con un ACK, continua procesando pedidos.
- Si el servidor no le responde con un ACK, el pedido se guarda en la lista de pedidos a sincronizar.
  
3. Procesa pedido.

4.1 Envía mensaje **COMPLETE** al servidor: para actualizar los puntos y desbloquear la cuenta del cliente si el pedido se pudo procesar correctamente.

5.1 Recibe mensaje **ACK** del servidor: si lo recibe continua con el procesamiento de otro pedido. Caso contrario, va a intentar enviar el mensaje una cantidad determinada de veces:

- Si el servidor responde con un ACK, continua procesando pedidos.
- Si el servidor no le responde con un ACK, el mensaje se guarda en la lista de mensajes a enviar al servidor cuando se vuelva a conectar en la red.

4.2 Envía mensaje **FAILURE** al servidor: para desbloquear la cuenta del cliente si el pedido se pudo procesar correctamente.

5.2 Recibe mensaje **ACK** del servidor: si lo recibe continua con el procesamiento de otro pedido. Caso contrario, va a intentar enviar el mensaje una cantidad determinada de veces:

- Si el servidor responde con un ACK, continua procesando pedidos.
- Si el servidor no le responde con un ACK, el mensaje se guarda en la lista de mensajes a enviar al servidor cuando se vuelva a conectar en la red.

### Caso: El cliente quiere pagar con puntos pero no tiene suficientes puntos

![tp2-concu-Sec  2 drawio](https://github.com/concurrentes-fiuba/2023-1c-tp2-concu-csv/assets/67125933/0e14652e-fbbf-4732-a953-577744993c0e)

### Caso: El cliente puede pagar con puntos o dinero pero se pierde el ACK del bloqueo de la cuenta del lider al servidor local

![tp2-concu-Sec  3 drawio](https://github.com/concurrentes-fiuba/2023-1c-tp2-concu-csv/assets/67125933/e11c6e1c-0c54-4d3b-befb-13c5c1b6282c)

## **Hipótesis**

- Los servidores locales no se caen permanentemente.

## **Ejecución del Programa**

Para ejecutar cada servidor local es necesario correr:

```cargo run --bin local_server <id de la sucursal> <cantidad total de sucursales>```

Para ejecutar las cafeteras es necesario correr:
```cargo run --bin coffee_machine <archivo con ordenes> <id de la sucursal>```

## **Casos de Prueba**
