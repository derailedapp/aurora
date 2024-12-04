/*
Copyright (C) 2024 V.J. De Chico

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

import * as React from "react";
import { View, Text, TextInput, Button, Pressable } from "react-native";
import { Controller, useForm } from "react-hook-form";
import { loginUser } from "@/lib/api";
import AsyncStorage from "@react-native-async-storage/async-storage";
import { useRouter } from "expo-router";

export default function Login() {
  const {
    control,
    handleSubmit,
    formState: { errors },
  } = useForm({
    defaultValues: {
      password: "",
      email: "",
    },
  });

  const router = useRouter();

  const onSubmit = async (data: { password: string; email: string }) => {
    const resp = await loginUser(data.password, data.email);

    const d = await resp.json();
    await AsyncStorage.setItem("token", d.token);
    // @ts-ignore
    router.push("/users/(accounts)/@me");
  };

  return (
    <View className="bg-white dark:bg-not-quite-dark-blue flex flex-col justify-center items-center h-screen">
      <Text className="text-brand dark:text-white text-2xl font-bold max-w-xs font-main">
        Welcome to Derailed!
      </Text>
      <Text className="text-black dark:text-white text-xl font-light max-w-xs font-main pb-3">
        We're glad to have you coming back
      </Text>
      <View className="black-glass-50 dark:white-glass-50 gap-5 xl:gap-10 xl:p-10 p-7 rounded-2xl">
        <View className="gap-2 lg:gap-3 min-w-80 xl:min-w-96">
          <Text className="text-black dark:text-gray-300 text-lg">Email</Text>
          <Controller
            // @ts-ignore
            control={control}
            rules={{ required: true }}
            render={({ field: { onChange, onBlur, value } }) => (
              <TextInput
                className="text-xl dark:text-white placeholder:text-gray-300 p-2 rounded-lg bg-not-quite-dark-blue/70"
                placeholder="godsend@zeus.gods"
                onBlur={onBlur}
                onChange={onChange}
                value={value}
              />
            )}
            name="email"
          ></Controller>
        </View>

        <View className="gap-2 lg:gap-3">
          <Text className="text-black dark:text-gray-300 text-lg">
            Password
          </Text>
          <Controller
            // @ts-ignore
            control={control}
            rules={{ required: true }}
            render={({ field: { onChange, onBlur, value } }) => (
              <TextInput
                className="text-xl dark:text-white placeholder:text-gray-300 p-2 rounded-lg bg-not-quite-dark-blue/70"
                placeholder="lactatingpregnantlatinas"
                onBlur={onBlur}
                onChange={onChange}
                value={value}
              />
            )}
            name="password"
          ></Controller>
        </View>
        <Pressable
          className="bg-brand rounded-lg p-2"
          onPress={handleSubmit(onSubmit)}
        >
          <Text className="text-not-quite-dark-blue dark:text-white text-center text-lg">
            Login
          </Text>
        </Pressable>
      </View>
    </View>
  );
}
