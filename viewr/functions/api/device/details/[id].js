export async function onRequestGet(context) {
  const device_id = context.params.id;

  if (!device_id) {
    return new Response('Device with that id was not found', { status: 404 })
  }

  const details = await context.env.DEVICE_DETAILS.get(device_id);

  return new Response(details);
}