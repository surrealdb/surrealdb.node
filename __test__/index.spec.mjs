import test from "ava";

import { Surreal } from 'surrealdb.js';
import { surrealdbNodeEngines } from '../lib-src/embedded.ts';

test("Connect in-memory SurrealDB instance", async (t) => {
	const db = new Surreal({
		engines: surrealdbNodeEngines(),
	});
	await db.connect("memory");
	t.pass();
});

test("set ns/db", async (t) => {
	{
		const db = new Surreal({
			engines: surrealdbNodeEngines(),
		});
		await db.connect("memory");
		await db.use({ namespace: "test" });
	}

	{
		const db = new Surreal({
			engines: surrealdbNodeEngines(),
		});
		await db.connect("memory");
		await db.use({ database: "test" });
	}

	{
		const db = new Surreal({
			engines: surrealdbNodeEngines(),
		});
		await db.connect("memory");
		await db.use({ namespace: "test", database: "test" });
	}

	t.pass();
});

test("test query method", async (t) => {
	const db = new Surreal({
		engines: surrealdbNodeEngines(),
	});
	await db.connect("memory");
	await db.use({ namespace: "test", database: "test" });

	{
		const [res] = await db.query("SELECT * FROM person");
		t.deepEqual(res, []);
	}

	{
		const [res] = await db.query("CREATE |foo:100|");
		t.is(res.length, 100);
	}

	{
		const data = { first_name: "Tobie", projects: ["SurrealDB"] };
		const [res] = await db.query("CREATE person:tobie content $data", {
			data: data,
		});
		data.id = "person:tobie";
		t.deepEqual(res, [data]);
	}
});

test("set and and unset", async (t) => {
	const db = new Surreal({
		engines: surrealdbNodeEngines(),
	});
	await db.connect("memory");
	await db.use({ namespace: "test", database: "test" });

	const data = { first: "Tobie", last: "Morgan Hitchcock" };

	await db.set("name", data);
	{
		const [res] = await db.query("RETURN $name");
		t.deepEqual(res, [data]);
	}

	await db.unset("name");

	{
		const [res] = await db.query("RETURN $name");
		t.deepEqual(res, []);
	}
});

test("auth", async (t) => {
	const db = new Surreal({
		engines: surrealdbNodeEngines(),
	});
	await db.connect("memory");
	await db.use({
		namespace: "test",
		database: "test"
	});

	const scope = /* surql */ `DEFINE SCOPE user SESSION 5s SIGNUP (CREATE type::thing('user', $username) SET email = $email, pass = crypto::argon2::generate($pass)) SIGNIN (SELECT * FROM user WHERE email = $email AND crypto::argon2::compare(pass, $pass))`;
	const [_, [{ scopes }]] = await db.query(/* surql */ `
		${scope};
		INFO FOR DB;
  	`);
	t.is(scopes.user, scope);

	{
		const token = await db.signup({
			namespace: "test",
			database: "test",
			scope: "user",

			username: 'john',
			email: "john.doe@example.com",
			pass: "password123",
		});
		t.is(typeof token, 'string');

		const [[user_id]] = await db.query(/* surql */ `$auth`);
		t.is(user_id, 'user:john');
	}

	{
		const token = await db.signin({
			namespace: "test",
			database: "test",
			scope: "user",

			email: "john.doe@example.com",
			pass: "password123",
		});
		t.is(typeof token, 'string');

		const [[user_id]] = await db.query(/* surql */ `$auth`);
		t.is(user_id, 'user:john');
	}
});

test("test select method", async (t) => {
	const db = new Surreal({
		engines: surrealdbNodeEngines(),
	});
	await db.connect("memory");
	await db.use({ namespace: "test", database: "test" });

	const jason = { id: "person:jason" };
	const john = { id: "person:john" };
	const jaime = { id: "person:jaime" };
	const people = [jaime, jason, john];

	await db.create(jason.id);
	await db.create(john.id);
	await db.create(jaime.id);

	{
		const res = await db.select("person");
		t.deepEqual(new Set(res), new Set(people));
		const person = await db.select("person:jason");
		t.deepEqual(person, [jason]);
	}
});

// test('examples', async t => {

//   async function query_and_print(db, query_str) {
//     let res = await db.query(query_str);
//     console.log(res);
//   }

//   const db_mem = new Surreal();
//   await db_mem.connect("memory");

//   const db_ws = new Surreal();
//   await db_ws.connect("ws://127.0.0.1:8000");

//   await db_mem.use({ ns: 'test', db: 'test' });
//   {
//     await db_mem.set('name', { first: 'Tobie', last: 'Morgan Hitchcock' });
//     await query_and_print(db_mem, "RETURN $name");

//     await db_mem.unset('name');
//     await query_and_print(db_mem, "RETURN $name");
//   }
//   await db_ws.signin({ username: 'root', password: 'root' });

//   // const scope = await db_ws.query(`DEFINE SCOPE user_scope SESSION 5s
//   //                     SIGNUP (CREATE user SET email = $email, pass = crypto::argon2::generate($pass) )
//   //                     SIGNIN (SELECT * FROM user WHERE email = $email AND crypto::argon2::compare(pass, $pass))
//   //                     `);
//   // console.log(scope);

//   // const token = await db_ws.signup({
//   //     namespace: 'namespace',
//   //     database: 'database',
//   //     scope: 'user_scope',
//   //     email: 'john.doe@example.com',
//   //     password: 'password123'
//   // });
//   // console.log(token);

//   // const token2 = await db_ws.signin({
//   //     namespace: 'namespace',
//   //     database: 'database',
//   //     scope: 'user_scope',
//   //     email: 'john.doe@example.com',
//   //     password: 'password123'
//   // });
//   // console.log(token2);

//   // await db.invalidate();

//   // await db.authenticate(token);

//   {
//     const query1 = await db_mem.query("CREATE person:raphael");
//     console.log(query1);

//     const query2 = await db_mem.query('SELECT * FROM type::table($table)', { table: 'person' });
//     console.log(query2);
//   }
//   {
//     const select1 = await db_mem.select('person');
//     console.log(select1);

//     const select2 = await db_mem.select('person:raphael');
//     console.log(select2);
//   }
//   {
//     let create1 = await db_mem.create("foo", { "test": "foo" });
//     console.log(create1);

//     const person1 = await db_mem.create('person');
//     console.log(person1);

//     const person2 = await db_mem.create('person', {
//       name: 'Tobie',
//       settings: {
//         active: true,
//         marketing: true
//       }
//     });
//     console.log(person2);
//   }

//   {
//     const people1 = await db_mem.update('person', {
//       name: 'Tobie',
//       settings: {
//         active: true,
//         marketing: true
//       }
//     });
//     console.log(people1);

//     await db_mem.create('person:janf');

//     // Replace a range of records with the specified data.
//     const person1 = await db_mem.update('person:jane..john', {
//       name: 'Tobie',
//       settings: {
//         active: true,
//         marketing: true
//       }
//     });
//     console.log(person1);

//     // Replace the current document / record data with the specified data.
//     const person2 = await db_mem.update('person:tobie', {
//       name: 'Tobie',
//       settings: {
//         active: true,
//         marketing: true
//       }
//     });
//     console.log(person2);

//   }

//   {
//     // Merge all records in a table with specified data.
//     const person1 = await db_mem.merge('person', {
//       marketing: true
//     });
//     console.log(person1);

//     // Merge a range of records with the specified data.
//     const person2 = await db_mem.merge('person:jane..john', {
//       marketing: true
//     });
//     console.log(person2);

//     // Merge the current document / record data with the specified data.
//     const person3 = await db_mem.merge('person:tobie', {
//       marketing: true
//     });
//     console.log(person3);
//   }

//   {
//     // Apply JSON Patch changes to all records in the database.
//     const person1 = await db_mem.patch('person', [{
//       op: 'replace',
//       path: '/settings/active',
//       value: false
//     }]);
//     console.log(person1);

//     // Apply JSON Patch to a range of records.
//     const person2 = await db_mem.patch('person:jane..john', [{
//       op: 'replace',
//       path: '/settings/active',
//       value: false
//     }]);
//     console.log(person2);

//     // Apply JSON Patch to a specific record.
//     const person3 = await db_mem.patch('person:tobie', [{
//       op: 'replace',
//       path: '/settings/active',
//       value: false
//     }]);
//     console.log(person3);
//   }

//   {
//     // Delete all records from a table
//     const records = await db_mem.delete('person');
//     console.log(records);

//     // Delete a range records from a table
//     const people = await db_mem.delete('person:jane..john');
//     console.log(people);

//     // Delete a specific record from a table
//     const record = await db_mem.delete('person:h5wxrf2ewk8xjxosxtyc');
//     console.log(record);
//   }

//   {
//     const version = await db_mem.version();
//     console.log(version);
//   }

//   {
//     await db_mem.health();
//   }

//   t.pass();
// })
