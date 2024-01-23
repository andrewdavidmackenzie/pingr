export async function onRequest(context) {
  const devices = await context.env.DEVICE_STATUS.list();
  console.log(`Devices:`);
  console.log(devices.keys);
  return new Response(devices.keys);
}