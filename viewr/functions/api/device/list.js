export async function onRequestGet(context) {
  const devices = await context.env.DEVICE_STATUS.list();
  return Response.json(devices.keys);
}