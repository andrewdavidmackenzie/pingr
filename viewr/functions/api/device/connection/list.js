export async function onRequestGet(context) {
  const devices = await context.env.DEVICE_ID_CONNECTION_MAPPING.list();

  var mappings;
  devices.keys.forEach((device_id) => {
    const connection = context.env.DEVICE_ID_CONNECTION_MAPPING.get(device_id);
    const mapping = {"device_id" : device_id, "connection": connection};
    mappings.push(mapping);
  })

  return Response.json(mappings);
}