export async function onRequestGet(context) {
  const connection_devices = await context.env.CONNECTION_DEVICE_STATUS.list();
  return Response.json(connection_devices.keys);
}