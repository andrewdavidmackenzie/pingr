export async function onRequestGet(context) {
  const device_id = context.params.id;

  if (!device_id) {
    return new Response('Not found', { status: 404 })
  }

  const status = await context.env.DEVICE_ID_CONNECTION_MAPPING.get(device_id);

  if (!status) {
    return new Response('Not found', { status: 404 })
  }

  return Response.json(status);
}