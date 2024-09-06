# Future Features for MetaSSR
This document outlines potential features and improvements that can be implemented in MetaSSR to enhance its functionality and performance. These ideas are aimed at making the framework more versatile, user-friendly, and robust.

## Ideas
### 1. **MetaSSR Node Module**

- **Description:** MetaSSR can provide a dedicated Node module that offers utility functions for React.js developers, making it easier to manage props, URL parameters, and server-side actions seamlessly. This module will abstract common SSR tasks, allowing developers to focus on building features rather than wiring up SSR infrastructure. Key utilities such as `useProps`, and `useQueryParams`.

- **Client-Side Example (React.js):**

    ```js
    // ./src/pages/groups/$group_id.js/$user_id.js
    import { useProps, useQueryParam } from '@metassr/hooks';

    export default function User() {
        // Fetch the props passed from the server (e.g., /groups/{group_id}/{user_id}?post_id=32)
        const { group_id, user_id } = useProps();
        const { post_id } = useQueryParams();

        return (
            <div>
                <h1>User ID: {user_id}</h1>
                <h1>Group ID: {group_id}</h1>
                <h1>Post ID: {post_id}</h1>
            </div>
        );
    };
    ```


- **Benefits:**
    - **Simplified SSR Logic:** These utility functions will hide the complexity of server-side rendering, letting developers focus on UI components.
    - **Seamless Integration with React.js:** Hooks like `useProps`, and `useQueryParams` will follow React's familiar hook-based architecture, making it easy to integrate into any React application.




## 2. **Middlewares**

- **Description:** Implement middleware support in MetaSSR to allow developers to intercept and modify requests and responses during the server-side rendering (SSR) process. This feature would enable custom logic to be applied at various points in the request lifecycle, such as authentication, logging, caching, or request transformation.


- **Polyglot Programming Support:**  
  MetaSSR will leverage the **Metacall** polyglot runtime, allowing middleware to be written in various programming languages like Python, JavaScript, or Ruby. This means developers can define middleware in their preferred language and easily integrate it into their application.

- **Example in Javascript:**
    ```js
    // ./src/middlewares/logger.js
    export function handler(req) {
        console.log(`[LOG_FROM_JS]: Request received at: ${new Date()}`);
    }
    ```
  **Example in Python:**
  ```python
  # ./src/middlewares/logger.py

  def handler(req):
      print(f"[LOG_FROM_PY]: Request received at: {datetime.now()}")
  ```

- **Benefits:**
  - Provides flexibility in handling requests, allowing for centralized control over functionalities like security, session management, and performance optimization.
  - Simplifies the addition of cross-cutting concernis, such as authentication or logging, without duplicating code across different routes or components.
  - Polyglot support allows developers to write middleware in multiple languages, making it adaptable for diverse use cases.



## 3. **Server Actions**

- **Description:** Introduce the concept of *server actions* to enable direct function execution on the server from the client side without needing to create traditional API routes. Developers can define server-side logic in their React components and call these server actions from the client.
  
- **Benefits:**
  - Simplifies communication between the client and server by allowing server-side functions to be invoked directly from the client.
  - Reduces boilerplate for API creation, making it easier to manage server-side logic within components.
  - Improves data fetching and mutation patterns, leading to more seamless integration of server-side and client-side operations.

- **Example**
    ```js
    // ./src/pages/home.jsx

    import { useAction } from 'metassr/hooks'; // Import the useAction hook from MetaSSR's node module

    // Home page component
    export default function Home() {
        // Hook to fetch results from server actions
        const { all, GET, POST, PUT, DELETE } = useAction();

        // Example usage of action results
        console.log('All actions:', all);
        console.log('GET action result:', GET);
        console.log('POST action result:', POST);
        console.log('PUT action result:', PUT);
        console.log('DELETE action result:', DELETE);

        return (
            <div>
                <h1>Hello World</h1>
                <p>Data from server actions will be logged in the console.</p>
            </div>
        );
    }

    // Default server action to handle general operations
    export function action() {
        // Perform operations on the server for any request
        // Example logging message (will appear in server logs)
        console.log('Executing default action on server');
        const result = { message: 'Default action result' };
        return result;
    }

    // Action for handling GET requests
    export function action$GET() {
        // Perform operations specific to GET requests
        // Example logging message (will appear in server logs)
        console.log('Executing GET action on server');
        const result = { message: 'GET request result' };
        return result;
    }

    // Action for handling POST requests
    export function action$POST() {
        // Perform operations specific to POST requests
        // Example logging message (will appear in server logs)
        console.log('Executing POST action on server');
        const result = { message: 'POST request result' };
        return result;
    }

    // Action for handling PUT requests
    export function action$PUT() {
        // Perform operations specific to PUT requests
        // Example logging message (will appear in server logs)
        console.log('Executing PUT action on server');
        const result = { message: 'PUT request result' };
        return result;
    }

    // Action for handling DELETE requests
    export function action$DELETE() {
        // Perform operations specific to DELETE requests
        // Example logging message (will appear in server logs)
        console.log('Executing DELETE action on server');
        const result = { message: 'DELETE request result' };
        return result;
    }


    ```


## 4. **API Routes**

- **Description:** Introduce API route support in MetaSSR to allow developers to define backend API endpoints directly within the framework. This feature would enable seamless integration of API logic alongside server-side rendering, allowing for dynamic content fetching, user authentication, or data manipulation without the need for external APIs.

- **Client-Side Example:**

    ```js
    // ./src/api/user.js

    export async function GET(req) {
        const { id } = req.params;
        const user = await database.getUserById(id);

        

        if (user) {
            res.status(200).json(user);
            return {
                status: 200,
                body: Json.stringify(user)
            }
        } else {
            return {
                status: 404,
                body: JSON.stringify({
                    message: "User is not found."
                })
            }
        }
    }

    export async function POST(req) {
        const newUser = await database.createUser(req.body);
        return {
            status: 201,
            body: Json.stringify(newUser)
        }
       
    }
    ```

- **Polyglot Programming Support:**
  With the integration of **Metacall**, API routes can be written in different programming languages, making it easy for developers to use their language of choice to define routes and business logic. This provides flexibility for teams with varying language expertise.

  **Example in Python:**
  ```python
    # ./src/api/user.py

    async def GET(req):
        user_id = req.params['id']
        user = await database.get_user_by_id(user_id)

        if user:
            return {
                "status": 200,
                "body": json.dumps(user)
            }
        else:
            return {
                "status": 404,
                "body": json.dumps({
                    "message": "User not found."
                })
            }

    async def POST(req):
        new_user = await database.create_user(req.body)
        return {
            "status": 201,
            "body": json.dumps(new_user)
        }
  ```

- **Benefits:**
  - Streamlines the process of building backend APIs for server-side rendered applications by co-locating API logic within the same framework.
  - Reduces the need for external services, enabling tighter integration between SSR components and backend API logic.
  - Polyglot support allows backend APIs to be written in different languages, giving developers the flexibility to use the best tools for their particular use case.

