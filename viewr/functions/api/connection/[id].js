export async function onRequestGet(context) {
  const connection_device_id = context.params.id;

  if (!connection_device_id) {
    return new Response('Not found', { status: 404 })
  }

  const status = await context.env.CONNECTION_DEVICE_STATUS.get(connection_device_id);

  if (!status) {
    return new Response('Not found', { status: 404 })
  }

  // TODO maybe simplify all these methods to returns Strings instead of json and simplify parsing?
  return Response.json(status);
}